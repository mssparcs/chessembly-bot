use std::cmp::Ordering;
use std::{collections::HashMap, hash::Hash};
mod behavior;
pub mod jit_compiler;
pub mod board;
pub mod moves;
use behavior::{Behavior};
pub(crate) use board::Board;
use serde::Serialize;
mod game_script;

use crate::chessembly::behavior::BehaviorChain;
use crate::chessembly::jit_compiler::{ChessemblyJitCompiler, CompiledChain};

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum GameResult {
    WhiteCheckmates,
    WhiteResigns,
    BlackCheckmates,
    BlackResigns,
    Stalemate,
    DrawAccepted,
    DrawDeclared,
}

#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline]
    pub fn invert(&self) -> Color {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
    
    #[inline]
    pub fn i8d(&self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }    
    }

    #[inline]
    pub fn i8v2(&self, dx: i8, dy: i8) -> DeltaPosition {
        match self {
            Self::White => (dx, dy),
            Self::Black => (-dx, -dy),
        }    
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece<'a> {
    pub piece_type: &'a str,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceSpan<'a> {
    Piece(Piece<'a>),
    Empty,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, serde::Serialize)]
pub enum MoveType {
    Move,
    TakeMove,
    Take,
    TakeJump,
    Catch,
    Shift,
    Castling

    // Void, Pause, Block
}

pub type Position = (u8, u8);
pub type DeltaPosition = (i8, i8);

#[derive(Clone, Eq, PartialOrd, PartialEq, Debug, Hash, Serialize)]
pub struct ChessMoveUnit<'a> {
    pub from: Position,
    pub take: Position,
    pub move_to: Position,
    pub move_type: MoveType,
    pub state_change: Option<Vec<(&'a str, u8)>>,
    pub transition: Option<&'a str>,
}

#[derive(Clone, Eq, PartialOrd, PartialEq, Debug, Hash, Serialize)]
pub enum ChessMove<'a> {
    Single(ChessMoveUnit<'a>),
    Multiple(Vec<ChessMoveUnit<'a>>)
}

impl<'a> ChessMove<'a> {
    #[inline]
    pub fn get_source(&self) -> Position {
        match self {
            ChessMove::Single(n) => n.from,
            ChessMove::Multiple(v) => v[0].from
        }
    }

    // Get the destination square (square the piece is going to).
    #[inline]
    pub fn get_dest(&self) -> Position {
        match self {
            ChessMove::Single(n) => n.move_to,
            ChessMove::Multiple(v) => v[0].move_to
        }
    }

    // Get the promotion piece (maybe).
    #[inline]
    pub fn get_promotion(&self) -> &Option<&'a str> {
        match self {
            ChessMove::Single(n) => &n.transition,
            ChessMove::Multiple(v) => &v[0].transition
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChessemblyCompiled {
    pub compiled_chains: Vec<CompiledChain>,
    pub precompiled_units: Vec<CompiledChain>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy, PartialEq)]
enum WallCollision {
    EdgeTop,
    EdgeBottom,
    EdgeLeft,
    EdgeRight,
    CornerTopLeft,
    CornerTopRight,
    CornerBottomLeft,
    CornerBottomRight,
    NoCollision,
}

pub struct MoveGen {}

impl MoveGen {
    pub fn get_all_moves<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(board: &mut Board<'a, MACHO, IMPRISONED, SIZE>, turn: Color, check_danger: bool) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        for j in 0..board.get_height() {
            for i in 0..board.get_width() {
                if board.color_on(&(i as u8, j as u8)) == Some(turn) {
                    if check_danger || MACHO {
                        let a = board
                            .script
                            .get_moves::<MACHO, IMPRISONED, SIZE>(board, &(i as u8, j as u8), check_danger);
                        let b = board.script.filter_nodes::<MACHO, IMPRISONED, SIZE>(a, board);
                        ret.extend(b);
                    } else {
                        ret.extend(board.script.get_moves::<MACHO, IMPRISONED, SIZE>(
                            board,
                            &(i as u8, j as u8),
                            check_danger,
                        ));
                    }
                }
            }
        }
        ret
    }

    pub fn has_any_moves<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(board: &mut Board<'a, MACHO, IMPRISONED, SIZE>, turn: Color, check_danger: bool) -> bool {
        for j in 0..board.get_height() {
            for i in 0..board.get_width() {
                if board.color_on(&(i as u8, j as u8)) == Some(turn) {
                    if check_danger || MACHO {
                        let a = board
                            .script
                            .get_moves::<MACHO, IMPRISONED, SIZE>(board, &(i as u8, j as u8), check_danger);
                        let b = board.script.filter_nodes::<MACHO, IMPRISONED, SIZE>(a, board);
                        if !b.is_empty() {
                            return true;
                        }
                    } else {
                        if !board.script.get_moves::<MACHO, IMPRISONED, SIZE>(
                            board,
                            &(i as u8, j as u8),
                            check_danger,
                        ).is_empty() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    #[inline]
    pub fn new_legal<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(board: &mut Board<'a, MACHO, IMPRISONED, SIZE>) -> Vec<ChessMove<'a>> {
        MoveGen::get_all_moves::<MACHO, IMPRISONED, SIZE>(board, board.side_to_move(), true)
    }

    #[inline]
    pub fn get_danger_zones<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(board: &mut Board<MACHO, IMPRISONED, SIZE>, enemy: Color) -> Vec<Position> {
        let mut ret = Vec::new();
        let all_moves = MoveGen::get_all_moves::<MACHO, IMPRISONED, SIZE>(board, enemy, false);
        for node in all_moves {
            match node {
                ChessMove::Multiple(v) => {
                    ret.extend(v.iter()
                        .filter(|x| match x.move_type {
                            MoveType::Take => true,
                            MoveType::TakeMove => true,
                            MoveType::TakeJump => true,
                            MoveType::Catch => true,
                            _ => false,
                        })
                        .map(|x| x.take));
                },
                ChessMove::Single(n) => {
                    match n.move_type {
                        MoveType::Take => ret.push(n.take),
                        MoveType::TakeMove => ret.push(n.take),
                        MoveType::TakeJump => ret.push(n.take),
                        MoveType::Catch => ret.push(n.take),
                        _ => ()
                    }
                }
            }
        }

        ret
    }

    pub fn get_danger_zones_bit<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(board: &mut Board<MACHO, IMPRISONED, SIZE>, enemy: Color) -> u64 {
        let mut ret: u64 = 0;
        let all_moves = MoveGen::get_all_moves::<MACHO, IMPRISONED, SIZE>(board, enemy, false);
        for node in all_moves {
            ret |= match node {
                ChessMove::Multiple(v) => {
                    v.iter().map(|x| (match x.move_type {
                        MoveType::Take => 1,
                        MoveType::TakeMove => 1,
                        MoveType::TakeJump => 1,
                        MoveType::Catch => 1,
                        _ => 0,
                    }) << (x.take.1 * 8 + x.take.0)).fold(0, |a, b| a | b)
                },
                ChessMove::Single(n) => {
                    match n.move_type {
                        MoveType::Take => 1 << (n.take.1 * 8 + n.take.0),
                        MoveType::TakeMove => 1 << (n.take.1 * 8 + n.take.0),
                        MoveType::TakeJump => 1 << (n.take.1 * 8 + n.take.0),
                        MoveType::Catch => 1 << (n.take.1 * 8 + n.take.0),
                        _ => 0
                    }
                }
            };
        }
        ret
    }
}

impl ChessemblyCompiled {
    pub fn new() -> ChessemblyCompiled {
        let mut ret = ChessemblyCompiled { compiled_chains: Vec::new(), precompiled_units: Vec::new() };
        let units_chain = ChessemblyCompiled::from_script_to_chain(game_script::GAME_SCRIPT).unwrap();
        for chain in units_chain {
            let mut compiler = ChessemblyJitCompiler::new();
            let compiled = compiler.compile::<false, false, 8>(&chain);
            ret.precompiled_units.push(compiled);
        }
        ret
    }

    #[inline]
    pub fn push_compiled(&mut self, compiled: CompiledChain) {
        self.compiled_chains.push(compiled);
    }

    pub fn from_chains_vec<'a>(chains: Vec<BehaviorChain<'a>>) -> ChessemblyCompiled {
        let mut ret = ChessemblyCompiled::new();
        for chain in chains {
            let mut compiler = ChessemblyJitCompiler::new();
            let compiled = compiler.compile::<false, false, 8>(&chain);
            ret.push_compiled(compiled);
        }
        ret
    }

    pub fn from_script_to_chain<'a>(script: &'a str) -> Result<Vec<BehaviorChain<'a>>, ()> {
        let mut ret = Vec::new();
        let chains = script.split(';');
        for chain_str in chains {
            if chain_str.trim().starts_with('#') {
                continue;
            } else if chain_str.chars().all(char::is_whitespace) {
                continue;
            } else {
                let mut chain = Vec::new();
                let mut i = 0;
                let mut j = 0;
                while j < chain_str.len() - 1 {
                    let jp1 = chain_str.ceil_char_boundary(j + 1);
                    if chain_str[j..jp1].chars().all(char::is_whitespace) {
                        let jp2 = chain_str.ceil_char_boundary(jp1 + 1);
                        if chain_str[jp1..jp2]
                            .chars()
                            .all(|c| char::is_alphabetic(c) || c == '{' || c == '}')
                        {
                            if chain_str[i..j].trim().len() > 0 {
                                chain.push(Behavior::from_str(&chain_str[i..j].trim()));
                                i = j;
                            }
                        }
                    }
                    j = jp1;
                }
                if !chain_str[i..].chars().all(char::is_whitespace) {
                    chain.push(Behavior::from_str(&chain_str[i..].trim()));
                }

                ret.push(chain);
            }
        }
        Ok(ret)
    
    }
    pub fn from_script(script: &str) -> Result<ChessemblyCompiled, ()> {
        let mut ret = ChessemblyCompiled::new();
        let chains = script.split(';');
        for chain_str in chains {
            if chain_str.trim().starts_with('#') {
                continue;
            } else if chain_str.chars().all(char::is_whitespace) {
                continue;
            } else {
                let mut chain = Vec::new();
                let mut i = 0;
                let mut j = 0;
                while j < chain_str.len() - 1 {
                    let jp1 = chain_str.ceil_char_boundary(j + 1);
                    if chain_str[j..jp1].chars().all(char::is_whitespace) {
                        let jp2 = chain_str.ceil_char_boundary(jp1 + 1);
                        if chain_str[jp1..jp2]
                            .chars()
                            .all(|c| char::is_alphabetic(c) || c == '{' || c == '}')
                        {
                            if chain_str[i..j].trim().len() > 0 {
                                chain.push(Behavior::from_str(&chain_str[i..j].trim()));
                                i = j;
                            }
                        }
                    }
                    j = jp1;
                }
                if !chain_str[i..].chars().all(char::is_whitespace) {
                    chain.push(Behavior::from_str(&chain_str[i..].trim()));
                }

                let mut compiler = ChessemblyJitCompiler::new();
                let compiled = compiler.compile::<false, false, 8>(&chain);
                ret.push_compiled(compiled);
            }
        }
        Ok(ret)
    }

    fn wall_collision<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(anchor: &Position, delta: &DeltaPosition, board: &Board<MACHO, IMPRISONED, SIZE>, color: Color) -> WallCollision {
        let a0 = (anchor.0 as i8) + delta.0;
        let a1 = (anchor.1 as i8) - delta.1;
        match (a0.cmp(&0), a0.cmp(&(board.get_width() as i8)), a1.cmp(&0), a1.cmp(&(board.get_height() as i8))) {
            (Ordering::Less, _, Ordering::Less, _) => if color == Color::White { WallCollision::CornerTopLeft } else { WallCollision::CornerBottomRight }
            (_, Ordering::Equal, Ordering::Less, _) => if color == Color::White { WallCollision::CornerTopRight } else { WallCollision::CornerBottomLeft }
            (_, Ordering::Greater, Ordering::Less, _) => if color == Color::White { WallCollision::CornerTopRight } else { WallCollision::CornerBottomLeft }
            (Ordering::Less, _, _, Ordering::Equal) => if color == Color::White { WallCollision::CornerBottomLeft } else { WallCollision::CornerTopRight }
            (Ordering::Less, _, _, Ordering::Greater) => if color == Color::White { WallCollision::CornerBottomLeft } else { WallCollision::CornerTopRight }
            (_, Ordering::Equal, _, Ordering::Equal) => if color == Color::White { WallCollision::CornerBottomRight } else { WallCollision::CornerTopLeft }
            (_, Ordering::Greater, _, Ordering::Greater) => if color == Color::White { WallCollision::CornerBottomRight } else { WallCollision::CornerTopLeft }
            (Ordering::Less, _, _, _) => if color == Color::White { WallCollision::EdgeLeft } else { WallCollision::EdgeRight }
            (_, Ordering::Equal, _, _) => if color == Color::White { WallCollision::EdgeRight } else { WallCollision::EdgeLeft }
            (_, Ordering::Greater, _, _) => if color == Color::White { WallCollision::EdgeRight } else { WallCollision::EdgeLeft }
            (_, _, Ordering::Less, _) => if color == Color::White { WallCollision::EdgeTop } else { WallCollision::EdgeBottom }
            (_, _, _, Ordering::Equal) => if color == Color::White { WallCollision::EdgeBottom } else { WallCollision::EdgeTop }
            (_, _, _, Ordering::Greater) => if color == Color::White { WallCollision::EdgeBottom } else { WallCollision::EdgeTop }
            _ => WallCollision::NoCollision
        }
    }

    fn move_anchor<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(anchor: &mut Position, delta: &DeltaPosition, board: &Board<MACHO, IMPRISONED, SIZE>, color: Color) -> WallCollision {
        let wc = ChessemblyCompiled::wall_collision(anchor, delta, board, color);
        if wc == WallCollision::NoCollision {
            anchor.0 = ((anchor.0 as i8) + delta.0) as u8;
            anchor.1 = ((anchor.1 as i8) - delta.1) as u8;
            return WallCollision::NoCollision;
        }
        wc
    }

    pub fn cancel_move_anchor(anchor: &mut Position, delta: &DeltaPosition) {
        anchor.0 = ((anchor.0 as i8) - delta.0) as u8;
        anchor.1 = ((anchor.1 as i8) + delta.1) as u8;
    }

    pub fn is_enemy<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(anchor: &Position, board: &Board<MACHO, IMPRISONED, SIZE>, color: Color) -> bool {
        if board.color_on(anchor) == Some(color.invert()) {
            return true;
        }
        false
    }

    pub fn is_friendly<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(anchor: &Position, board: &Board<MACHO, IMPRISONED, SIZE>, color: Color) -> bool {
        if board.color_on(anchor) == Some(color) {
            return true;
        }
        false
    }
    
    pub fn is_zero_vector(delta: &DeltaPosition) -> bool {
        delta.0 == 0 && delta.1 == 0
    }

    pub fn is_danger<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, board: &mut Board<MACHO, IMPRISONED, SIZE>, position: &Position, color: Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones_bit::<MACHO, IMPRISONED, SIZE>(board, color);
        ChessemblyCompiled::is_danger_bit(danger_zones, position.0, position.1)
    }

    pub fn is_danger_bit(danger_zones_bit: u64, x: u8, y: u8) -> bool {
        (danger_zones_bit & (1 << (8 * y + x))) != 0
    }

    pub fn is_check<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, board: &mut Board<MACHO, IMPRISONED, SIZE>, color: Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones::<MACHO, IMPRISONED, SIZE>(board, color);
        danger_zones
            .iter()
            .any(|x| board.piece_on(x) == Some("king"))
    }

    pub fn is_check_dbg<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, board: &mut Board<MACHO, IMPRISONED, SIZE>, color: Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones::<MACHO, IMPRISONED, SIZE>(board, color);
        println!("------------------------ {:?}", color.invert());
        for i in 0..8 {
            let mut x = String::new();
            for j in 0..8 {
                if danger_zones.contains(&(j, i)) {
                    x.push_str(
                        &format!(
                            "[{}]",
                            board
                                .piece_on(&(j, i))
                                .map(|x| x.chars().next().unwrap())
                                .unwrap_or(' ')
                        )[..],
                    );
                } else {
                    x.push_str(
                        &format!(
                            " {} ",
                            board
                                .piece_on(&(j, i))
                                .map(|x| x.chars().next().unwrap())
                                .unwrap_or(' ')
                        )[..],
                    );
                }
            }
            println!("{}", x);
        }
        let ret = danger_zones
            .iter()
            .any(|x| board.piece_on(x) == Some("king"));

        if ret {
            println!("==================> Check!")
        }
        else {
            println!("==================> OK")
        }
        ret
    }

    pub fn push_node<'a>(nodes: &mut Vec<ChessMove<'a>>, node: ChessMoveUnit<'a>) {
        if let Some(i) = nodes
            .iter()
            .position(|x| x.get_dest() == node.move_to && match x {
                ChessMove::Single(n) => n.take,
                ChessMove::Multiple(v) => v[0].take,
            } == node.take)
        {
            nodes.swap_remove(i);
        }
        nodes.push(ChessMove::Single(node));
    }

    pub fn generate_moves<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(
        &self,
        board: &mut Board<'a, MACHO, IMPRISONED, SIZE>,
        position: &Position,
        check_danger: bool,
    ) -> Result<Vec<ChessMove<'a>>, ()> {
        let mut nodes: Vec<ChessMove> = Vec::new();

        for compiled in &self.compiled_chains {
            if let Some(_) = board.color_on(position) {
                compiled.execute_from(&board, *position, &mut nodes, check_danger);
            }
            else {
                return Err(());
            }
        }

        for compiled in &self.precompiled_units {
            if let Some(_) = board.color_on(position) {
                compiled.execute_from(&board, *position, &mut nodes, check_danger);
            }
            else {
                return Err(());
            }
        }

        Ok(nodes)
    }

    pub fn filter_nodes<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, nodes: Vec<ChessMove<'a>>, board: &Board<'a, MACHO, IMPRISONED, SIZE>) -> Vec<ChessMove<'a>> {
        let mut ret: Vec<ChessMove> = Vec::new();
        if MACHO {
            for testnode in nodes {
                let piece_color = board.color_on(&testnode.get_source()).unwrap();
                match (testnode.get_source().1.cmp(&testnode.get_dest().1), piece_color) {
                    (Ordering::Less, Color::Black) => ret.push(testnode),
                    (Ordering::Greater, Color::White) => ret.push(testnode),
                    (Ordering::Equal, _) => {
                        if let ChessMove::Single(n) = testnode {
                            if board.color_on(&n.take) == Some(piece_color.invert()) {
                                ret.push(ChessMove::Single(ChessMoveUnit {
                                    from: n.from,
                                    take: n.take,
                                    move_to: n.move_to,
                                    move_type: MoveType::Take,
                                    state_change: n.state_change,
                                    transition: n.transition
                                }));
                            }
                        }
                        //
                    },
                    (_, _) => {}
                }
            }
            ret
        }
        else {
            for testnode in nodes {
                let mut new_board = board.make_move_new_nc(&testnode, false);
                let turn = new_board.turn;
                new_board.turn = new_board.turn.invert();
                if !self.is_check::<MACHO, IMPRISONED, SIZE>(&mut new_board, turn.invert()) {
                    ret.push(testnode);
                }
            }

            ret
        }
    }

    pub fn get_moves<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, board: &mut Board<'a, MACHO, IMPRISONED, SIZE>, position: &Position, check_danger: bool) -> Vec<ChessMove<'a>> {
        let piece_on = board.piece_on(position);
        let Some(piece) = piece_on else {
            return Vec::new()
        };
        // worker::console_log!("{}", piece);
        match piece {
            "pawn" => self.generate_pawn_moves::<MACHO, IMPRISONED, SIZE>(board, position),
            "king" => {
                let danger_zones = if check_danger { MoveGen::get_danger_zones_bit::<MACHO, IMPRISONED, SIZE>(board, board.color_on(position).unwrap().invert()) } else { 0 };
                let ret = self.generate_king_moves::<MACHO, IMPRISONED, SIZE>(board, position, danger_zones);
                ret
            },
            "rook" => self.generate_rook_moves::<MACHO, IMPRISONED, SIZE>(board, position),
            "beacon" => self.generate_beacon_moves::<MACHO, IMPRISONED, SIZE>(board, position),
            "chameleon" => self.generate_chameleon_moves::<MACHO, IMPRISONED, SIZE>(board, position),
            "mirrored-pawn" => self.generate_mirrored_moves::<MACHO, IMPRISONED, SIZE>(board, position, "mirrored-pawn"),
            "mirrored-bishop" => self.generate_mirrored_moves::<MACHO, IMPRISONED, SIZE>(board, position, "mirrored-bishop"),
            "mirrored-rook" => self.generate_mirrored_moves::<MACHO, IMPRISONED, SIZE>(board, position, "mirrored-rook"),
            "mirrored-knight" => self.generate_mirrored_moves::<MACHO, IMPRISONED, SIZE>(board, position, "mirrored-knight"),
            "mirrored-queen" => self.generate_mirrored_moves::<MACHO, IMPRISONED, SIZE>(board, position, "mirrored-queen"),
            _ => {
                let ret = self.generate_moves::<MACHO, IMPRISONED, SIZE>(board, position, check_danger);
                ret.unwrap_or(Vec::new())
            }
        }
    }
}
