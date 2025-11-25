use std::cmp::Ordering;
use std::{collections::HashMap, hash::Hash};
mod behavior;
pub mod board;
pub mod moves;
use behavior::{Behavior, BehaviorChain};
pub(crate) use board::Board;

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
    pub fn invert(&self) -> Color {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Piece<'a> {
    pub piece_type: &'a str,
    pub color: Color,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PieceSpan<'a> {
    Piece(Piece<'a>),
    Empty,
}

use serde::Serialize;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Serialize)]
pub enum MoveType {
    Move,
    TakeMove,
    Take,
    TakeJump,
    Catch,
    // Castling,

    // Void, Pause, Shift Block
}

pub type Position = (u8, u8);
pub type DeltaPosition = (i8, i8);

#[derive(Clone, Eq, PartialOrd, PartialEq, Debug, Hash, Serialize)]
pub struct ChessMove<'a> {
    pub from: Position,
    pub take: Position,
    pub move_to: Position,
    pub move_type: MoveType,
    pub state_change: Option<Vec<(&'a str, u8)>>,
    pub transition: Option<&'a str>,
}

impl<'a> ChessMove<'a> {
    #[inline]
    pub fn get_source(&self) -> Position {
        self.from
    }

    // Get the destination square (square the piece is going to).
    #[inline]
    pub fn get_dest(&self) -> Position {
        self.move_to
    }

    // Get the promotion piece (maybe).
    #[inline]
    pub fn get_promotion(&self) -> &Option<&'a str> {
        &self.transition
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChessemblyCompiled<'a> {
    pub chains: Vec<BehaviorChain<'a>>,
}

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
    pub fn get_all_moves<'a>(board: &mut Board<'a>, turn: Color, check_danger: bool) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        for j in 0..board.get_height() {
            for i in 0..board.get_width() {
                if board.color_on(&(i as u8, j as u8)) == Some(turn) {
                    if check_danger {
                        let a = board
                            .script
                            .get_moves(board, &(i as u8, j as u8), check_danger);
                        let b = board.script.filter_nodes(a, board);
                        ret.extend(b);
                    } else {
                        ret.extend(board.script.get_moves(
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

    #[inline]
    pub fn new_legal<'a>(board: &mut Board<'a>) -> Vec<ChessMove<'a>> {
        MoveGen::get_all_moves(board, board.side_to_move(), true)
    }

    #[inline]
    pub fn get_danger_zones(board: &mut Board, enemy: Color) -> Vec<Position> {
        MoveGen::get_all_moves(board, enemy, false)
            .iter()
            .map(|x| x.take)
            .collect()
    }
}

impl<'a> ChessemblyCompiled<'a> {
    pub fn new() -> ChessemblyCompiled<'a> {
        ChessemblyCompiled { chains: Vec::new() }
    }

    #[inline]
    pub fn add_command(&mut self) {
        self.chains.push(Vec::new());
    }

    #[inline]
    pub fn push_behavior(&mut self, behavior: Behavior<'a>) {
        let x = &mut self.chains.last_mut();
        if let Some(last) = x {
            last.push(behavior);
        }
    }

    pub fn from_script(script: &'a str) -> Result<ChessemblyCompiled<'a>, ()> {
        let mut ret = ChessemblyCompiled::new();
        let chains = script.split(';');
        for chain_str in chains {
            if chain_str.trim().starts_with('#') {
                continue;
            } else if chain_str.chars().all(char::is_whitespace) {
                continue;
            } else {
                ret.add_command();
                let mut i = 0;
                let mut j = 0;
                while j < chain_str.len() - 1 {
                    if chain_str[j..j + 1].chars().all(char::is_whitespace) {
                        if chain_str[j + 1..j + 2]
                            .chars()
                            .all(|c| char::is_alphabetic(c) || c == '{' || c == '}')
                        {
                            if chain_str[i..j].trim().len() > 0 {
                                ret.push_behavior(Behavior::from_str(&chain_str[i..j].trim()));
                                i = j;
                            }
                        }
                    }
                    j += 1;
                }
                if !chain_str[i..].chars().all(char::is_whitespace) {
                    ret.push_behavior(Behavior::from_str(&chain_str[i..].trim()));
                }
            }
        }
        Ok(ret)
    }

    fn wall_collision(anchor: &Position, delta: &DeltaPosition, board: &Board, color: Color) -> WallCollision {
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

    fn move_anchor(anchor: &mut Position, delta: &DeltaPosition, board: &Board, color: Color) -> WallCollision {
        let wc = ChessemblyCompiled::wall_collision(anchor, delta, board, color);
        if wc == WallCollision::NoCollision {
            anchor.0 = ((anchor.0 as i8) + delta.0) as u8;
            anchor.1 = ((anchor.1 as i8) - delta.1) as u8;
            return WallCollision::NoCollision;
        }
        wc
    }

    fn cancel_move_anchor(anchor: &mut Position, delta: &DeltaPosition) {
        anchor.0 = ((anchor.0 as i8) - delta.0) as u8;
        anchor.1 = ((anchor.1 as i8) + delta.1) as u8;
    }

    fn is_enemy(anchor: &Position, board: &Board, color: Color) -> bool {
        if board.color_on(anchor) == Some(color.invert()) {
            return true;
        }
        false
    }

    fn is_friendly(anchor: &Position, board: &Board, color: Color) -> bool {
        if board.color_on(anchor) == Some(color) {
            return true;
        }
        false
    }

    pub fn is_zero_vector(delta: &DeltaPosition) -> bool {
        delta.0 == 0 && delta.1 == 0
    }

    pub fn is_danger(&self, board: &mut Board, position: &Position, color: Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones(board, color);
        danger_zones.iter().any(|x| x == position)
    }

    pub fn is_check(&self, board: &mut Board, color: Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones(board, color);
        danger_zones
            .iter()
            .any(|x| board.piece_on(x) == Some("king"))
    }

    pub fn is_check_dbg(&self, board: &mut Board, color: Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones(board, color);
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
        }
        danger_zones
            .iter()
            .any(|x| board.piece_on(x) == Some("king"))
    }

    pub fn push_node(nodes: &mut Vec<ChessMove<'a>>, node: ChessMove<'a>) {
        if let Some(i) = nodes
            .iter()
            .position(|x| x.move_to == node.move_to && x.take == node.take)
        {
            nodes.swap_remove(i);
        }
        nodes.push(node);
    }

    pub fn generate_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
        check_danger: bool,
    ) -> Result<Vec<ChessMove<'a>>, ()> {
        let mut nodes: Vec<ChessMove> = Vec::new();

        for chain in &self.chains {
            let mut rip: usize = 0;
            let mut loops = 0;
            let mut stack: Vec<(Position, usize)> = vec![(position.clone(), chain.len())];
            let mut take_stack: Vec<Option<Position>> = vec![None];
            let mut states: Vec<bool> = vec![true];
            let mut transition: Option<*const str> = None;
            let mut state_change: Option<Vec<(*const str, u8)>> = None;

            while rip < chain.len() {
                let abs_inst = &chain[rip];
                loops += 1;
                if loops > 1000 {
                    break;
                }

                let is_control_expr = match abs_inst {
                    Behavior::While => true,
                    Behavior::Jmp(_) => true,
                    Behavior::Jne(_) => true,
                    Behavior::Label(_) => true,
                    Behavior::Not => true,
                    _ => false,
                };

                if *states.last().unwrap() == false && !is_control_expr {
                    if stack.len() > 1 {
                        rip = stack.last().unwrap().1;
                    } else {
                        break;
                    }
                }

                let abs_inst = &chain[rip];
                let inst = abs_inst.reflect_turn(board.side_to_move());

                if stack.len() == 0 || states.len() == 0 {
                    break;
                }

                match inst {
                    Behavior::TakeMove(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );

                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::cancel_move_anchor(
                                &mut stack.last_mut().unwrap().0,
                                &delta,
                            );
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        } else if ChessemblyCompiled::is_enemy(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::push_node(
                                &mut nodes,
                                ChessMove {
                                    from: position.clone(),
                                    take: stack.last_mut().unwrap().0.clone(),
                                    move_to: stack.last_mut().unwrap().0.clone(),
                                    move_type: MoveType::TakeMove,
                                    state_change: state_change.clone().map(|x| {
                                        x.iter()
                                            .map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v))
                                            .collect()
                                    }),
                                    transition: transition.map(|x| unsafe { x.as_ref().unwrap() }),
                                },
                            );
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        } else {
                            ChessemblyCompiled::push_node(
                                &mut nodes,
                                ChessMove {
                                    from: position.clone(),
                                    take: stack.last_mut().unwrap().0.clone(),
                                    move_to: stack.last_mut().unwrap().0.clone(),
                                    move_type: MoveType::TakeMove,
                                    state_change: state_change.clone().map(|x| {
                                        x.iter()
                                            .map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v))
                                            .collect()
                                    }),
                                    transition: transition.map(|x| unsafe { x.as_ref().unwrap() }),
                                },
                            );
                            rip += 1;
                        }
                    }
                    Behavior::BlockOpen => {
                        let mut end = rip;
                        let mut ss = 0;
                        while end < chain.len() {
                            match &chain[end] {
                                Behavior::BlockOpen => {
                                    ss += 1;
                                }
                                Behavior::BlockClose => {
                                    ss -= 1;
                                    if ss == 0 {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            end += 1;
                        }
                        stack.push((stack.last().unwrap().clone().0, end));
                        if let Some(p) = take_stack.last() {
                            take_stack.push(p.clone());
                        } else {
                            take_stack.push(None);
                        }
                        states.push(true);
                        rip += 1;
                    }
                    Behavior::BlockClose => {
                        if stack.len() > 1 && states.len() > 1 {
                            stack.pop();
                            states.pop();
                        } else {
                            break;
                        }
                        rip += 1;
                    }
                    Behavior::Peek(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );

                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if let Some(_) = board.color_on(&stack.last().unwrap().0) {
                            ChessemblyCompiled::cancel_move_anchor(
                                &mut stack.last_mut().unwrap().0,
                                &delta,
                            );
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        rip += 1;
                    }
                    Behavior::Observe(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );

                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if let Some(_) = board.color_on(&stack.last().unwrap().0) {
                            *states.last_mut().unwrap() = false;
                        }
                        ChessemblyCompiled::cancel_move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                        );
                        rip += 1;
                        continue;
                    }
                    Behavior::Piece(piece_name) => {
                        if let Some(piece) = board.piece_on(position) {
                            // worker::console_log!("{}, {}", piece_name, piece);
                            *states.last_mut().unwrap() = piece == piece_name;
                        } else {
                            *states.last_mut().unwrap() = false;
                        }
                        rip += 1;
                    }
                    Behavior::Bound(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = match wc {
                            WallCollision::NoCollision => false,
                            _ => true,
                        };
                        rip += 1;
                    }
                    Behavior::Edge(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = match wc {
                            WallCollision::EdgeTop => true,
                            WallCollision::EdgeBottom => true,
                            WallCollision::EdgeLeft => true,
                            WallCollision::EdgeRight => true,
                            _ => false,
                        };
                        rip += 1;
                    }
                    Behavior::Corner(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = match wc {
                            WallCollision::CornerTopLeft => true,
                            WallCollision::CornerTopRight => true,
                            WallCollision::CornerBottomLeft => true,
                            WallCollision::CornerBottomRight => true,
                            _ => false,
                        };
                        rip += 1;
                    }
                    Behavior::EdgeTop(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeTop;
                        rip += 1;
                    }
                    Behavior::EdgeBottom(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeBottom;
                        rip += 1;
                    }
                    Behavior::EdgeLeft(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeLeft;
                        rip += 1;
                    }
                    Behavior::EdgeRight(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeRight;
                        rip += 1;
                    }
                    Behavior::CornerTopLeft(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::CornerTopLeft;
                        rip += 1;
                    }
                    Behavior::CornerTopRight(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::CornerTopRight;
                        rip += 1;
                    }
                    Behavior::CornerBottomLeft(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::CornerBottomLeft;
                        rip += 1;
                    }
                    Behavior::CornerBottomRight(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(
                            &stack.last().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        *states.last_mut().unwrap() = wc == WallCollision::CornerBottomRight;
                        rip += 1;
                    }
                    Behavior::Check => {
                        *states.last_mut().unwrap() =
                            self.is_check(board, board.color_on(position).unwrap());
                        rip += 1;
                    }
                    Behavior::Danger(delta) => {
                        if !check_danger {
                            *states.last_mut().unwrap() = false;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        *states.last_mut().unwrap() = self.is_danger(
                            board,
                            &stack.last().unwrap().0,
                            board.color_on(position).unwrap(),
                        );
                        ChessemblyCompiled::cancel_move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                        );
                    }
                    Behavior::Enemy(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        *states.last_mut().unwrap() = ChessemblyCompiled::is_enemy(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        ChessemblyCompiled::cancel_move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                        );
                        rip += 1;
                    }
                    Behavior::Friendly(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        *states.last_mut().unwrap() = ChessemblyCompiled::is_friendly(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        ChessemblyCompiled::cancel_move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                        );
                        rip += 1;
                    }
                    Behavior::PieceOn((piece_name, delta)) => {
                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        *states.last_mut().unwrap() =
                            board.piece_on(&stack.last().unwrap().0) == Some(&piece_name[..]);
                        ChessemblyCompiled::cancel_move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                        );
                        rip += 1;
                    }
                    Behavior::IfState((key, n)) => {
                        if board.color_on(position) == Some(Color::White) {
                            *states.last_mut().unwrap() =
                                *board.board_state.white.register.get(key).unwrap_or(&0) == n;
                        } else if board.color_on(position) == Some(Color::Black) {
                            *states.last_mut().unwrap() =
                                *board.board_state.black.register.get(key).unwrap_or(&0) == n;
                        }
                        rip += 1;
                    }
                    Behavior::SetState((key, n)) => {
                        if let Some(state_changes) = &mut state_change {
                            state_changes.push((key, n));
                        } else {
                            state_change = Some(vec![(key, n)]);
                        }
                        rip += 1;
                    }
                    Behavior::Transition(piece_name) => {
                        if piece_name.len() == 0 {
                            transition = None;
                        } else {
                            transition = Some(piece_name);
                        }
                        rip += 1;
                    }
                    Behavior::Take(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::cancel_move_anchor(
                                &mut stack.last_mut().unwrap().0,
                                &delta,
                            );
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        } else if ChessemblyCompiled::is_enemy(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::push_node(
                                &mut nodes,
                                ChessMove {
                                    from: position.clone(),
                                    take: stack.last().unwrap().0.clone(),
                                    move_to: stack.last().unwrap().0.clone(),
                                    move_type: MoveType::Take,
                                    state_change: state_change.clone().map(|x| {
                                        x.iter()
                                            .map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v))
                                            .collect()
                                    }),
                                    transition: transition.map(|x| unsafe { x.as_ref().unwrap() }),
                                },
                            );
                            if let Some(_) = take_stack.pop() {
                                take_stack.push(Some(stack.last().unwrap().0.clone()));
                            } else {
                                take_stack.push(Some(stack.last().unwrap().0.clone()));
                            }
                        }
                        rip += 1;
                    }
                    Behavior::Jump(delta) => {
                        let tl1 = take_stack.last();
                        if let Some(tp) = tl1 {
                            if let Some(tpc) = tp {
                                if let Some(trace) = nodes
                                    .iter()
                                    .position(|x| x.move_type == MoveType::Take && x.take == *tpc)
                                {
                                    nodes.swap_remove(trace);
                                }

                                if !ChessemblyCompiled::is_zero_vector(&delta) {
                                    let wc = ChessemblyCompiled::move_anchor(
                                        &mut stack.last_mut().unwrap().0,
                                        &delta,
                                        board,
                                        board.color_on(position).unwrap(),
                                    );
                                    if wc == WallCollision::NoCollision {
                                        if board.color_on(&stack.last().unwrap().0).is_none() {
                                            ChessemblyCompiled::push_node(
                                                &mut nodes,
                                                ChessMove {
                                                    from: position.clone(),
                                                    take: tpc.clone(),
                                                    move_to: stack.last().unwrap().0.clone(),
                                                    move_type: MoveType::TakeJump,
                                                    state_change: state_change.clone().map(|x| {
                                                        x.iter()
                                                            .map(|(k, v)| {
                                                                (unsafe { k.as_ref().unwrap() }, *v)
                                                            })
                                                            .collect()
                                                    }),
                                                    transition: transition
                                                        .map(|x| unsafe { x.as_ref().unwrap() }),
                                                },
                                            );
                                            rip += 1;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }

                        *states.last_mut().unwrap() = false;
                        rip += 1;
                        continue;
                    }
                    Behavior::Catch(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );

                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::cancel_move_anchor(
                                &mut stack.last_mut().unwrap().0,
                                &delta,
                            );
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        } else if ChessemblyCompiled::is_enemy(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::push_node(
                                &mut nodes,
                                ChessMove {
                                    from: position.clone(),
                                    take: stack.last_mut().unwrap().0.clone(),
                                    move_to: position.clone(),
                                    move_type: MoveType::Catch,
                                    state_change: state_change.clone().map(|x| {
                                        x.iter()
                                            .map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v))
                                            .collect()
                                    }),
                                    transition: transition.map(|x| unsafe { x.as_ref().unwrap() }),
                                },
                            );
                        }
                        rip += 1;
                    }
                    Behavior::Move(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(
                            &mut stack.last_mut().unwrap().0,
                            &delta,
                            board,
                            board.color_on(position).unwrap(),
                        );

                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::cancel_move_anchor(
                                &mut stack.last_mut().unwrap().0,
                                &delta,
                            );
                            *states.last_mut().unwrap() = false;
                        } else if ChessemblyCompiled::is_enemy(
                            &stack.last().unwrap().0,
                            board,
                            board.color_on(position).unwrap(),
                        ) {
                            ChessemblyCompiled::cancel_move_anchor(
                                &mut stack.last_mut().unwrap().0,
                                &delta,
                            );
                            *states.last_mut().unwrap() = false;
                        } else {
                            ChessemblyCompiled::push_node(
                                &mut nodes,
                                ChessMove {
                                    from: position.clone(),
                                    take: stack.last_mut().unwrap().0.clone(),
                                    move_to: stack.last_mut().unwrap().0.clone(),
                                    move_type: MoveType::Move,
                                    state_change: state_change.clone().map(|x| {
                                        x.iter()
                                            .map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v))
                                            .collect()
                                    }),
                                    transition: transition.map(|x| unsafe { x.as_ref().unwrap() }),
                                },
                            );
                        }
                        rip += 1;
                    }
                    Behavior::Repeat(n) => {
                        if n == 0 {
                            break;
                        }
                        if n as usize > rip {
                            break;
                        }
                        rip -= n as usize;
                    }
                    Behavior::Not => {
                        let x = *states.last().unwrap();
                        *states.last_mut().unwrap() = !x;
                        rip += 1;
                    }
                    Behavior::Do => {
                        if let Some(next_inst) = chain.get(rip + 1) {
                            match next_inst {
                                Behavior::While => {
                                    rip += 1;
                                }
                                _ => {
                                    states.push(true);
                                }
                            }
                        } else {
                            break;
                        }
                        rip += 1;
                    }
                    Behavior::While => {
                        if *states.last().unwrap() {
                            let mut ss = 0;
                            loop {
                                if chain[rip] == Behavior::While {
                                    ss += 1;
                                } else if chain[rip] == Behavior::Do {
                                    ss -= 1;
                                    if ss == 0 {
                                        // ??
                                        break;
                                    }
                                }
                                if rip == 0 {
                                    break;
                                }
                                rip -= 1;
                            }
                        } else {
                            states.pop();
                            if states.len() == 0 {
                                break;
                            }
                            rip += 1;
                        }
                    }
                    Behavior::Label(_) => {
                        rip += 1;
                    }
                    Behavior::Jmp(label) => {
                        if *states.last().unwrap() {
                            if let Some(label_rip) = chain
                                .iter()
                                .enumerate()
                                .find(|&(_, v)| *v == Behavior::Label(label))
                            {
                                rip = label_rip.0;
                            } else {
                                break;
                            }
                        } else {
                            rip += 1;
                            *states.last_mut().unwrap() = true;
                        }
                    }
                    Behavior::Jne(label) => {
                        if !*states.last().unwrap() {
                            if let Some(label_rip) = chain
                                .iter()
                                .enumerate()
                                .find(|&(_, v)| *v == Behavior::Label(label))
                            {
                                rip = label_rip.0;
                            } else {
                                break;
                            }
                        } else {
                            rip += 1;
                            *states.last_mut().unwrap() = true;
                        }
                    }
                    _ => break,
                };
            }
        }
        return Ok(nodes);
    }

    pub fn filter_nodes(&self, nodes: Vec<ChessMove<'a>>, board: &Board<'a>) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        for testnode in nodes {
            let mut new_board = board.make_move_new_nc(&testnode, false);
            let turn = new_board.turn.invert();
            if !self.is_check(&mut new_board, turn) {
                ret.push(testnode);
            }
        }

        ret
    }

    pub fn get_moves(&self, board: &mut Board<'a>, position: &Position, check_danger: bool) -> Vec<ChessMove<'a>> {
        if let Some(cached) = board.dp.get(position) {
            return cached.clone();
        }

        let piece_on = board.piece_on(position);
        let Some(piece) = piece_on else {
            return Vec::new()
        };
        // worker::console_log!("{}", piece);
        match piece {
            "pawn" => {
                let ret = self.generate_pawn_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "king" => {
                let danger_zones = if check_danger { MoveGen::get_danger_zones(board, board.color_on(position).unwrap().invert()) } else { Vec::new() };
                let ret = self.generate_king_moves(board, position, &danger_zones);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "rook" => {
                let ret = self.generate_rook_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "knight" => {
                let ret = self.generate_knight_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "bishop" => {
                let ret = self.generate_bishop_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "queen" => {
                let ret = self.generate_queen_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "tempest-rook" => {
                let ret = self.generate_tempest_rook_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "bouncing-bishop" => {
                let ret = self.generate_bouncing_bishop_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "dozer" => {
                let ret = self.generate_dozer_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "alfil" => {
                let ret = self.generate_alfil_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "bard" => {
                let ret = self.generate_bard_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "zebra" => {
                let ret = self.generate_ij_moves(board, position, 3, 2);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "giraffe" => {
                let ret = self.generate_ij_moves(board, position, 4, 1);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            "camel" => {
                let ret = self.generate_ij_moves(board, position, 3, 1);
                board.dp.insert((position.0, position.1), ret.clone());
                ret
            }
            _ => {
                let ret = self.generate_moves(board, position, check_danger);
                board.dp.insert((position.0, position.1), ret.clone().unwrap_or(Vec::new()));
                ret.unwrap_or(Vec::new())
            }
        }
    }
}
