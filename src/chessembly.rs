use std::{collections::HashMap, hash::Hash};
mod moves;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Eq)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

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

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BoardState<'a> {
    pub castling_oo :bool,
    pub castling_ooo :bool,
    pub enpassant :Vec<Position>,
    pub register :HashMap<&'a str, u8>
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BothBoardState<'a> {
    pub black :BoardState<'a>,
    pub white :BoardState<'a>
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
            Self::Black => Self::White
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Piece<'a> {
    pub piece_type :&'a str,
    pub color :Color,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PieceSpan<'a> {
    Piece(Piece<'a>),
    Empty
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board<'a> {
    pub board :[[PieceSpan<'a>; 8]; 8],
    pub board_state :BothBoardState<'a>,
    pub turn :Color,
    pub script :&'a ChessemblyCompiled<'a>,
    pub status :BoardStatus,
    dp :HashMap<Position, Vec<ChessMove<'a>>>
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
    pub from :Position,
    pub take :Position,
    pub move_to :Position,
    pub move_type :MoveType,
    pub state_change :Option<Vec<(&'a str, u8)>>,
    pub transition :Option<&'a str>
}

impl<'a> ChessMove<'a> {
    /// Create a new chess move, given a source `Square`, a destination `Square`, and an optional
    /// promotion `Piece`
    // #[inline]
    // pub fn new() -> ChessMove {
    //     ChessMove {
            
    //     }
    // }

    #[inline]
    pub fn get_source(&self) -> Position {
        self.from
    }

    /// Get the destination square (square the piece is going to).
    #[inline]
    pub fn get_dest(&self) -> Position {
        self.move_to
    }

    /// Get the promotion piece (maybe).
    #[inline]
    pub fn get_promotion(&self) -> &Option<&'a str> {
        &self.transition
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Behavior<'a> {
    TakeMove(DeltaPosition),
    Take(DeltaPosition),
    Repeat(i8),
    Move(DeltaPosition),
    Catch(DeltaPosition),
    // Void(Position),
    Peek(DeltaPosition),
    Observe(DeltaPosition),
    While,
    Jump(DeltaPosition),
    Do,
    Bound(DeltaPosition),
    Edge(DeltaPosition),
    EdgeTop(DeltaPosition),
    EdgeLeft(DeltaPosition),
    EdgeRight(DeltaPosition),
    EdgeBottom(DeltaPosition),
    Corner(DeltaPosition),
    CornerTopLeft(DeltaPosition),
    CornerTopRight(DeltaPosition),
    CornerBottomLeft(DeltaPosition),
    CornerBottomRight(DeltaPosition),
    Not,
    Jmp(u8),
    Jne(u8),
    BlockOpen,
    BlockClose,
    Label(u8),
    End,
    Danger(DeltaPosition),
    Check,
    Enemy(DeltaPosition),
    Friendly(DeltaPosition),
    PieceOn((&'a str, DeltaPosition)),
    SetState((&'a str, u8)),
    IfState((&'a str, u8)),
    Transition(&'a str),
    Piece(String),
}

pub type BehaviorChain<'a> = Vec<Behavior<'a>>;

#[derive(Debug, PartialEq, Eq)]
pub struct ChessemblyCompiled<'a> {
    pub chains :Vec<BehaviorChain<'a>>
}

impl<'a> Board<'a> {
    pub fn from_str(placement :&str, script :&'a ChessemblyCompiled) -> Board<'a> {
        let mut ret = Board {
            dp: HashMap::new(),
            board: [
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty]
            ],
            board_state: BothBoardState {
                black: BoardState {
                    castling_oo: true, castling_ooo: true, enpassant: Vec::new(), register: HashMap::new()
                },
                white: BoardState {
                    castling_oo: true, castling_ooo: true, enpassant: Vec::new(), register: HashMap::new()
                }
            },
            script: script,
            turn: Color::White,
            status: BoardStatus::Ongoing
        };
        for i in 0..8 {
            for j in 0..8 {
                if placement.chars().nth((i * 9 + j) as usize) == Some('Q') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "queen", color: Color::White })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('N') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "knight", color: Color::White })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('K') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "king", color: Color::White })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('B') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "bishop", color: Color::White })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('R') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "rook", color: Color::White })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('P') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "pawn", color: Color::White })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('q') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "queen", color: Color::Black })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('n') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "knight", color: Color::Black })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('k') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "king", color: Color::Black })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('b') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "bishop", color: Color::Black })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('r') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "rook", color: Color::Black })
                }
                else if placement.chars().nth((i * 9 + j) as usize) == Some('p') {
                    ret.board[i][j] = PieceSpan::Piece(Piece { piece_type: "pawn", color: Color::Black })
                }
            }
        }
        ret
    }

    pub fn empty(script :&'a ChessemblyCompiled) -> Board<'a> {
        Board {
            dp: HashMap::new(),
            board: [
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty]
            ],
            board_state: BothBoardState {
                black: BoardState {
                    castling_oo: true, castling_ooo: true, enpassant: Vec::new(), register: HashMap::new()
                },
                white: BoardState {
                    castling_oo: true, castling_ooo: true, enpassant: Vec::new(), register: HashMap::new()
                }
            },
            script: script,
            turn: Color::White,
            status: BoardStatus::Ongoing
        }
    }

    pub fn new(script :&'a ChessemblyCompiled) -> Board<'a> {
        Board {
            dp: HashMap::new(),
            board: [
                [
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "rook" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "knight" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "bishop" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "queen" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "king" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "bishop" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "knight" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "rook" })
                ],
                [
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" })
                ],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
                [
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" })
                ],
                [
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "rook" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "knight" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "bishop" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "queen" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "king" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "bishop" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "knight" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "rook" })
                ]
            ],
            board_state: BothBoardState {
                black: BoardState {
                    castling_oo: true, castling_ooo: true, enpassant: Vec::new(), register: HashMap::new()
                },
                white: BoardState {
                    castling_oo: true, castling_ooo: true, enpassant: Vec::new(), register: HashMap::new()
                }
            },
            script: script,
            turn: Color::White,
            status: BoardStatus::Ongoing
        }
    }

    pub fn to_string(&self) -> String {
        let mut ret = String::new();
        for j in 0..8 {
            for i in 0..8 {
                if let Some(color) = self.color_on(&(i, j)) {
                    if self.piece_on(&(i, j)).unwrap() == "pawn" {
                        if color == Color::Black {
                            ret.push('p');
                        }
                        else if color == Color::White {
                            ret.push('P');
                        }
                    }
                    else if self.piece_on(&(i, j)).unwrap() == "rook" {
                        if color == Color::Black {
                            ret.push('r');
                        }
                        else if color == Color::White {
                            ret.push('R');
                        }
                    }
                    else if self.piece_on(&(i, j)).unwrap() == "bishop" {
                        if color == Color::Black {
                            ret.push('b');
                        }
                        else if color == Color::White {
                            ret.push('B');
                        }
                    }
                    else if self.piece_on(&(i, j)).unwrap() == "knight" {
                        if color == Color::Black {
                            ret.push('n');
                        }
                        else if color == Color::White {
                            ret.push('N');
                        }
                    }
                    else if self.piece_on(&(i, j)).unwrap() == "king" {
                        if color == Color::Black {
                            ret.push('k');
                        }
                        else if color == Color::White {
                            ret.push('K');
                        }
                    }
                    else if self.piece_on(&(i, j)).unwrap() == "queen" {
                        if color == Color::Black {
                            ret.push('q');
                        }
                        else if color == Color::White {
                            ret.push('Q');
                        }
                    }
                    else if let Some(piece) = self.piece_on(&(i, j)) {
                        ret.push(piece.chars().next().unwrap());
                    }
                }
                else {
                    ret.push(' ');
                }
            }
            ret.push('\n');
        }
        ret
    }

    pub fn make_move_new_nc(&self, node :&ChessMove<'a>, decide :bool) -> Board<'a> {
        let mut ret = self.clone();
        ret.dp = HashMap::new();
        ret.board[node.take.1 as usize][node.take.0 as usize] = PieceSpan::Empty;
        ret.board[node.move_to.1 as usize][node.move_to.0 as usize] = node.transition.as_ref().map(|x|
            PieceSpan::Piece(Piece {
                piece_type: x,
                color: match &ret.board[node.from.1 as usize][node.from.0 as usize] {
                    PieceSpan::Empty => Color::White,
                    PieceSpan::Piece(piece) => piece.color
                }
            })
        ).unwrap_or(ret.board[node.from.1 as usize][node.from.0 as usize].clone());
        ret.board[node.from.1 as usize][node.from.0 as usize] = PieceSpan::Empty;
        if let Some(state_changes) = &node.state_change {
            for (key, n) in state_changes {
                if key == &"castling-oo" {
                    if ret.turn == Color::White {
                        ret.board_state.white.castling_oo = *n > 0;
                    }
                    else if ret.turn == Color::Black {
                        ret.board_state.black.castling_oo = *n > 0;
                    }
                }
                else if key == &"castling-ooo" {
                    if ret.turn == Color::White {
                        ret.board_state.white.castling_ooo = *n > 0;
                    }
                    else if ret.turn == Color::Black {
                        ret.board_state.black.castling_ooo = *n > 0;
                    }
                }
                else if key == &"en-passant" {
                    if ret.turn == Color::White {
                        ret.board_state.black.enpassant.push(node.move_to.clone());
                    }
                    else if ret.turn == Color::Black {
                        ret.board_state.white.enpassant.push(node.move_to.clone());
                    }
                }
            }
        }
        
        if !decide {
            return ret;
        }

        ret.turn = ret.turn.invert();
        
        let turn = ret.side_to_move();
        if MoveGen::get_all_moves(&mut ret, turn, true).len() == 0 {
            if self.script.is_check(&mut ret, turn.invert()) {
                ret.status = BoardStatus::Checkmate;
            }
            else {
                ret.status = BoardStatus::Stalemate;
            }
        }
        ret
    }

    #[inline]
    pub fn make_move_new(&self, node :&ChessMove<'a>) -> Board<'a> {
        self.make_move_new_nc(node, true)
    }

    #[inline]
    pub fn status(&self) -> BoardStatus {
        self.status
    }

    #[inline]
    pub fn piece_on(&self, position :&Position) -> Option<&str> {
        if position.0 > 7 || position.1 > 7 {
            return None;
        }
        else if let PieceSpan::Piece(piece) = &self.board[position.1 as usize][position.0 as usize] {
            return Some(&piece.piece_type);
        }
        None
    }

    #[inline]
    pub fn color_on(&self, position :&Position) -> Option<Color> {
        if position.0 > 7 || position.1 > 7 {
            return None;
        }
        else if let PieceSpan::Piece(piece) = &self.board[position.1 as usize][position.0 as usize] {
            return Some(piece.color);
        }
        None
    }
    
    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.turn
    }
    
    #[inline]
    pub fn get_width(&self) -> usize {
        8
    }

    #[inline]
    pub fn get_height(&self) -> usize {
        8
    }
}

impl<'a> Behavior<'a> {
    pub fn from_str(fragment :&'a str) -> Behavior<'a> {
        if fragment.starts_with("end") {
            return Behavior::End;
        }
        else if fragment.starts_with("while") {
            return Behavior::While;
        }
        else if fragment.starts_with("do") {
            return Behavior::Do;
        }
        else if fragment.starts_with("not") {
            return Behavior::Not;
        }
        else if fragment.starts_with("check") {
            return Behavior::Check;
        }
        else if fragment == "transition" {
            return Behavior::Transition("");
        }
        else if fragment.starts_with("}") {
            return Behavior::BlockClose;
        }
        else if fragment.starts_with("{") {
            return Behavior::BlockOpen;
        }
        let fs1 = fragment.split_once('(');
        if fs1.is_none() {
            // worker::console_log!("??????");
            return Behavior::End;
        }
        let (cmd, pwr) = fs1.unwrap();
        let fs2 = pwr.split_once(')');
        if fs2.is_none() {
            // worker::console_log!("???????????");
            return Behavior::End;
        }
        let (params, _) = fs2.unwrap();
        let params_vec :Vec<&str> = params.split(',').map(|x| x.trim()).collect();

        if cmd == "label" {
            return Behavior::Label(params_vec.get(0).map(|s| s.parse::<u8>().unwrap_or(0)).unwrap_or(0));
        }
        else if cmd == "jmp" {
            return Behavior::Jmp(params_vec.get(0).map(|s| s.parse::<u8>().unwrap_or(0)).unwrap_or(0));
        }
        else if cmd == "jne" {
            return Behavior::Jne(params_vec.get(0).map(|s| s.parse::<u8>().unwrap_or(0)).unwrap_or(0));
        }
        else if cmd == "repeat" {
            return Behavior::Repeat(params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0));
        }
        else if cmd == "transition" {
            return Behavior::Transition(params_vec.get(0).unwrap_or(&""));
        }
        else if cmd == "piece" {
            return Behavior::Piece(params_vec.get(0).map(|s| String::from(*s)).unwrap_or(String::new()));
        }
        else if cmd == "set-state" {
            return Behavior::SetState((params_vec.get(0).unwrap_or(&""), params_vec.get(1).map(|s| s.parse::<u8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "if-state" {
            return Behavior::IfState((params_vec.get(0).unwrap_or(&""), params_vec.get(1).map(|s| s.parse::<u8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "piece-on" {
            return Behavior::PieceOn((params_vec.get(0).unwrap_or(&""), (
                params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0),
                params_vec.get(2).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)
            )));
        }
        else if cmd == "take-move" {
            return Behavior::TakeMove((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "take" {
            return Behavior::Take((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "jump" {
            return Behavior::Jump((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "move" {
            return Behavior::Move((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "catch" {
            return Behavior::Catch((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "danger" {
            return Behavior::Danger((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "enemy" {
            return Behavior::Enemy((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "friendly" {
            return Behavior::Friendly((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "peek" {
            return Behavior::Peek((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "observe" {
            return Behavior::Observe((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "bound" {
            return Behavior::Bound((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "edge" {
            return Behavior::Edge((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "corner" {
            return Behavior::Corner((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "edge-left" {
            return Behavior::EdgeLeft((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "edge-right" {
            return Behavior::EdgeRight((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "edge-top" {
            return Behavior::EdgeTop((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "edge-bottom" {
            return Behavior::EdgeBottom((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "corner-top-left" {
            return Behavior::CornerTopLeft((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "corner-top-right" {
            return Behavior::CornerTopRight((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "corner-bottom-left" {
            return Behavior::CornerBottomLeft((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        else if cmd == "corner-bottom-right" {
            return Behavior::CornerBottomRight((params_vec.get(0).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0), params_vec.get(1).map(|s| s.parse::<i8>().unwrap_or(0)).unwrap_or(0)));
        }
        
        // worker::console_log!("??????????");
        Behavior::End
    }

    fn reflect_turn_vector(position :&DeltaPosition, turn :Color) -> DeltaPosition {
        if turn == Color::Black {
            return (-position.0, -position.1);
        }
        else {
            return position.clone();
        }
    }

    pub fn reflect_turn(&'a self, turn :Color) -> Behavior<'a> {
        match self {
            Behavior::Bound(delta) => Behavior::Bound(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Edge(delta) => Behavior::Edge(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Corner(delta) => Behavior::Corner(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::EdgeTop(delta) => Behavior::EdgeTop(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::EdgeBottom(delta) => Behavior::EdgeBottom(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::EdgeLeft(delta) => Behavior::EdgeLeft(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::EdgeRight(delta) => Behavior::EdgeRight(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::CornerTopLeft(delta) => Behavior::CornerTopLeft(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::CornerTopRight(delta) => Behavior::CornerTopLeft(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::CornerBottomLeft(delta) => Behavior::CornerBottomLeft(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::CornerBottomRight(delta) => Behavior::CornerBottomRight(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Enemy(delta) => Behavior::Enemy(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Friendly(delta) => Behavior::Friendly(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Danger(delta) => Behavior::Danger(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Take(delta) => Behavior::Take(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Jump(delta) => Behavior::Jump(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::TakeMove(delta) => Behavior::TakeMove(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Move(delta) => Behavior::Move(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Catch(delta) => Behavior::Catch(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Observe(delta) => Behavior::Observe(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Peek(delta) => Behavior::Peek(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::PieceOn((piece, delta)) => Behavior::PieceOn((piece, Behavior::reflect_turn_vector(delta, turn))),
            _ => self.clone(),
        }
    }
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
    NoCollision
}

pub struct MoveGen {

}

impl MoveGen {
    pub fn get_all_moves<'a>(board :&mut Board<'a>, turn :Color, check_danger :bool) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        for j in 0..board.get_height() {
            for i in 0..board.get_width() {
                if board.color_on(&(i as u8, j as u8)) == Some(turn) {
                    if check_danger {
                        let a = board.script.get_moves(board, &(i as u8, j as u8), check_danger);
                        let b = board.script.filter_nodes(a, board);
                        ret.extend(b);
                    }
                    else {
                        ret.extend(board.script.get_moves(board, &(i as u8, j as u8), check_danger));
                    }
                }
            }
        }
        ret
    }

    #[inline]
    pub fn new_legal<'a>(board :&mut Board<'a>) -> Vec<ChessMove<'a>> {
        MoveGen::get_all_moves(board, board.side_to_move(), true)
    }

    #[inline]
    pub fn get_danger_zones(board :&mut Board, enemy :Color) -> Vec<Position> {
        MoveGen::get_all_moves(board, enemy, false).iter().map(|x| x.take).collect()
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
    pub fn push_behavior(&mut self, behavior :Behavior<'a>) {
        let x = &mut self.chains.last_mut();
        if let Some(last) = x {
            last.push(behavior);
        }
    }

    pub fn from_script(script :&'a str) -> Result<ChessemblyCompiled<'a>, ()> {
        let mut ret = ChessemblyCompiled::new();
        let chains = script.split(';');
        for chain_str in chains {
            if chain_str.trim().starts_with('#') {
                continue;
            }
            else if chain_str.chars().all(char::is_whitespace) {
                continue;
            }
            else {
                ret.add_command();
                let mut i = 0;
                let mut j = 0;
                while j < chain_str.len() - 1 {
                    if chain_str[j..j+1].chars().all(char::is_whitespace) {
                        if chain_str[j+1..j+2].chars().all(|c| char::is_alphabetic(c) || c == '{' || c == '}') {
                            if chain_str[i..j].trim().len() > 0 {
                                // worker::console_log!("{:?}", &chain_str[i..j].trim());
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

    fn wall_collision(anchor :&Position, delta :&DeltaPosition, board :&Board, color :Color) -> WallCollision {
        let a0 = (anchor.0 as i8) + delta.0;
        let a1 = (anchor.1 as i8) - delta.1;
        if a1 < 0 && a0 < 0 {
            return if color == Color::White { WallCollision::CornerTopLeft } else { WallCollision::CornerBottomRight };
        }
        if a1 < 0 && a0 >= (board.get_width() as i8) {
            return if color == Color::White { WallCollision::CornerTopRight } else { WallCollision::CornerBottomLeft };
        }
        if a1 >= (board.get_height() as i8) && a0 < 0 {
            return if color == Color::White { WallCollision::CornerBottomLeft } else { WallCollision::CornerTopRight };
        }
        if a1 >= (board.get_height() as i8) && a0 >= (board.get_width() as i8) {
            return if color == Color::White { WallCollision::CornerBottomRight } else { WallCollision::CornerTopLeft };
        }
        if a0 < 0 {
            return if color == Color::White { WallCollision::EdgeLeft } else { WallCollision::EdgeRight };
        }
        if a0 >= (board.get_width() as i8) {
            return if color == Color::White { WallCollision::EdgeRight } else { WallCollision::EdgeLeft };
        }
        if a1 < 0 {
            return if color == Color::White { WallCollision::EdgeTop } else { WallCollision::EdgeBottom };
        }
        if a1 >= (board.get_height() as i8) {
            return if color == Color::White { WallCollision::EdgeBottom } else { WallCollision::EdgeTop };
        }
        WallCollision::NoCollision
    }

    fn move_anchor(anchor :&mut Position, delta :&DeltaPosition, board :&Board, color :Color) -> WallCollision {
        let wc = ChessemblyCompiled::wall_collision(anchor, delta, board, color);
        if wc == WallCollision::NoCollision {
            anchor.0 = ((anchor.0 as i8) + delta.0) as u8;
            anchor.1 = ((anchor.1 as i8) - delta.1) as u8;
            return WallCollision::NoCollision;
        }
        wc
    }

    fn cancel_move_anchor(anchor :&mut Position, delta :&DeltaPosition) {
        anchor.0 = ((anchor.0 as i8) - delta.0) as u8;
        anchor.1 = ((anchor.1 as i8) + delta.1) as u8;
    }

    fn is_enemy(anchor :&Position, board :&Board, color :Color) -> bool {
        if board.color_on(anchor) == Some(color.invert()) {
            return true;
        }
        false
    }

    fn is_friendly(anchor :&Position, board :&Board, color :Color) -> bool {
        if board.color_on(anchor) == Some(color) {
            return true;
        }
        false
    }

    pub fn is_zero_vector(delta :&DeltaPosition) -> bool {
        delta.0 == 0 && delta.1 == 0
    }

    pub fn is_danger(&self, board :&mut Board, position :&Position, color :Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones(board, color);
        danger_zones.iter().any(|x| x == position)
    }

    pub fn is_check(&self, board :&mut Board, color :Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones(board, color);
        danger_zones.iter().any(|x| board.piece_on(x) == Some("king"))
    }

    pub fn is_check_dbg(&self, board :&mut Board, color :Color) -> bool {
        let danger_zones = MoveGen::get_danger_zones(board, color);
        for i in 0..8 {
            let mut x = String::new();
            for j in 0..8 {
                if danger_zones.contains(&(j, i)) {
                    x.push_str(&format!("[{}]", board.piece_on(&(j, i)).map(|x| x.chars().next().unwrap()).unwrap_or(' '))[..]);
                }
                else {
                    x.push_str(&format!(" {} ", board.piece_on(&(j, i)).map(|x| x.chars().next().unwrap()).unwrap_or(' '))[..]);
                }
            }
        }
        danger_zones.iter().any(|x| board.piece_on(x) == Some("king"))
    }

    pub fn push_node(nodes :&mut Vec<ChessMove<'a>>, node :ChessMove<'a>) {
        // if let Some(x) = node.transition {
            // worker::console_log!("{}", x);
        // }
        if let Some(i) = nodes.iter().position(|x| x.move_to == node.move_to && x.take == node.take) {
            nodes.swap_remove(i);
        }
        nodes.push(node);
    }

    pub fn generate_moves(&self, board :&mut Board<'a>, position :&Position, check_danger :bool) -> Result<Vec<ChessMove<'a>>, ()> {
        let mut nodes :Vec<ChessMove> = Vec::new();

        // TODO: 중복 노드 처리
        for chain in &self.chains {
            let mut rip :usize = 0;
            let mut loops = 0;
            let mut stack :Vec<(Position, usize)> = vec![(position.clone(), chain.len())];
            let mut take_stack :Vec<Option<Position>> = vec![None];
            let mut states :Vec<bool> = vec![true];
            let mut transition :Option<*const str> = None;
            let mut state_change :Option<Vec<(*const str, u8)>> = None;
            
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
                    _ => false
                };
                
                if *states.last().unwrap() == false && !is_control_expr {
                    if stack.len() > 1 {
                        rip = stack.last().unwrap().1;
                    }
                    else {
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

                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        else if ChessemblyCompiled::is_enemy(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::push_node(&mut nodes, ChessMove {
                                from: position.clone(),
                                take: stack.last_mut().unwrap().0.clone(),
                                move_to: stack.last_mut().unwrap().0.clone(),
                                move_type: MoveType::TakeMove,
                                state_change: state_change.clone().map(|x| x.iter().map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v)).collect()),
                                transition: transition.map(|x| unsafe { x.as_ref().unwrap() })
                            });
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        else {
                            ChessemblyCompiled::push_node(&mut nodes, ChessMove {
                                from: position.clone(),
                                take: stack.last_mut().unwrap().0.clone(),
                                move_to: stack.last_mut().unwrap().0.clone(),
                                move_type: MoveType::TakeMove,
                                state_change: state_change.clone().map(|x| x.iter().map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v)).collect()),
                                transition: transition.map(|x| unsafe { x.as_ref().unwrap() })
                            });
                            rip += 1;
                        }
                    },
                    Behavior::BlockOpen => {
                        let mut end = rip;
                        let mut ss = 0;
                        while end < chain.len() {
                            match &chain[end] {
                                Behavior::BlockOpen => {
                                    ss += 1;
                                },
                                Behavior::BlockClose => {
                                    ss -= 1;
                                    if ss == 0 {
                                        break;
                                    }
                                },
                                _ => {}
                            }
                            end += 1;
                        }
                        stack.push((stack.last().unwrap().clone().0, end));
                        if let Some(p) = take_stack.last() {
                            take_stack.push(p.clone());
                        }
                        else {
                            take_stack.push(None);
                        }
                        states.push(true);
                        rip += 1;
                    },
                    Behavior::BlockClose => {
                        if stack.len() > 1 && states.len() > 1 {
                            stack.pop();
                            states.pop();
                        }
                        else {
                            break;
                        }
                        rip += 1;
                    },
                    Behavior::Peek(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if let Some(_) = board.color_on(&stack.last().unwrap().0) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        rip += 1;
                    },
                    Behavior::Observe(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if let Some(_) = board.color_on(&stack.last().unwrap().0) {
                            *states.last_mut().unwrap() = false;
                        }
                        ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                        rip += 1;
                        continue;
                    },
                    Behavior::Piece(piece_name) => {
                        if let Some(piece) = board.piece_on(position) {
                            // worker::console_log!("{}, {}", piece_name, piece);
                            *states.last_mut().unwrap() = piece == piece_name;
                        }
                        else {
                            *states.last_mut().unwrap() = false;
                        }
                        rip += 1;
                    },
                    Behavior::Bound(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = match wc {
                            WallCollision::NoCollision => false,
                            _ => true
                        };
                        rip += 1;
                    },
                    Behavior::Edge(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = match wc {
                            WallCollision::EdgeTop => true,
                            WallCollision::EdgeBottom => true,
                            WallCollision::EdgeLeft => true,
                            WallCollision::EdgeRight => true,
                            _ => false
                        };
                        rip += 1;
                    },
                    Behavior::Corner(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = match wc {
                            WallCollision::CornerTopLeft => true,
                            WallCollision::CornerTopRight => true,
                            WallCollision::CornerBottomLeft => true,
                            WallCollision::CornerBottomRight => true,
                            _ => false
                        };
                        rip += 1;
                    },
                    Behavior::EdgeTop(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeTop;
                        rip += 1;
                    },
                    Behavior::EdgeBottom(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeBottom;
                        rip += 1;
                    },
                    Behavior::EdgeLeft(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeLeft;
                        rip += 1;
                    },
                    Behavior::EdgeRight(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::EdgeRight;
                        rip += 1;
                    },
                    Behavior::CornerTopLeft(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::CornerTopLeft;
                        rip += 1;
                    },
                    Behavior::CornerTopRight(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::CornerTopRight;
                        rip += 1;
                    },
                    Behavior::CornerBottomLeft(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::CornerBottomLeft;
                        rip += 1;
                    },
                    Behavior::CornerBottomRight(delta) => {
                        let wc = ChessemblyCompiled::wall_collision(&stack.last().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        *states.last_mut().unwrap() = wc == WallCollision::CornerBottomRight;
                        rip += 1;
                    },
                    Behavior::Check => {
                        *states.last_mut().unwrap() = self.is_check(board, board.color_on(position).unwrap());
                        rip += 1;
                    },
                    Behavior::Danger(delta) => {
                        if !check_danger {
                            *states.last_mut().unwrap() = false;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        *states.last_mut().unwrap() = self.is_danger(board, &stack.last().unwrap().0, board.color_on(position).unwrap());
                        ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                    },
                    Behavior::Enemy(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        *states.last_mut().unwrap() = ChessemblyCompiled::is_enemy(&stack.last().unwrap().0, board, board.color_on(position).unwrap());
                        ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                        rip += 1;
                    },
                    Behavior::Friendly(delta) => {
                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        *states.last_mut().unwrap() = ChessemblyCompiled::is_friendly(&stack.last().unwrap().0, board, board.color_on(position).unwrap());
                        ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                        rip += 1;
                    },
                    Behavior::PieceOn((piece_name, delta)) => {
                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        *states.last_mut().unwrap() = board.piece_on(&stack.last().unwrap().0) == Some(&piece_name[..]);
                        ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                        rip += 1;
                    },
                    Behavior::IfState((key, n)) => {
                        if board.color_on(position) == Some(Color::White) {
                            *states.last_mut().unwrap() = *board.board_state.white.register.get(key).unwrap_or(&0) == n;
                        }
                        else if board.color_on(position) == Some(Color::Black) {
                            *states.last_mut().unwrap() = *board.board_state.black.register.get(key).unwrap_or(&0) == n;
                        }
                        rip += 1;
                    },
                    Behavior::SetState((key, n)) => {
                        if let Some(state_changes) = &mut state_change {
                            state_changes.push((key, n));
                        }
                        else {
                            state_change = Some(vec![(key, n)]);
                        }
                        rip += 1;
                    },
                    Behavior::Transition(piece_name) => {
                        if piece_name.len() == 0 {
                            transition = None;
                        }
                        else {
                            transition = Some(piece_name);
                        }
                        rip += 1;
                    },
                    Behavior::Take(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        else if ChessemblyCompiled::is_enemy(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::push_node(&mut nodes, ChessMove {
                                from: position.clone(),
                                take: stack.last().unwrap().0.clone(),
                                move_to: stack.last().unwrap().0.clone(),
                                move_type: MoveType::Take,
                                state_change: state_change.clone().map(|x| x.iter().map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v)).collect()),
                                transition: transition.map(|x| unsafe { x.as_ref().unwrap() })
                            });
                            if let Some(_) = take_stack.pop() {
                                take_stack.push(Some(stack.last().unwrap().0.clone()));
                            }
                            else {
                                take_stack.push(Some(stack.last().unwrap().0.clone()));
                            }
                        }
                        rip += 1;
                    },
                    Behavior::Jump(delta) => {
                        // worker::console_log!("Jump");
                        let tl1 = take_stack.last();
                        if let Some(tp) = tl1 {
                            if let Some(tpc) = tp {
                                if let Some(trace) = nodes.iter().position(|x| x.move_type == MoveType::Take && x.take == *tpc) {
                                    nodes.swap_remove(trace);
                                }

                                if !ChessemblyCompiled::is_zero_vector(&delta) {
                                    let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                                    if wc == WallCollision::NoCollision {
                                        if board.color_on(&stack.last().unwrap().0).is_none() {
                                            ChessemblyCompiled::push_node(&mut nodes, ChessMove {
                                                from: position.clone(),
                                                take: tpc.clone(),
                                                move_to: stack.last().unwrap().0.clone(),
                                                move_type: MoveType::TakeJump,
                                                state_change: state_change.clone().map(|x| x.iter().map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v)).collect()),
                                                transition: transition.map(|x| unsafe { x.as_ref().unwrap() })
                                            });
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
                    },
                    Behavior::Catch(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;  
                        }
                        else if ChessemblyCompiled::is_enemy(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::push_node(&mut nodes, ChessMove {
                                from: position.clone(),
                                take: stack.last_mut().unwrap().0.clone(),
                                move_to: position.clone(),
                                move_type: MoveType::Catch,
                                state_change: state_change.clone().map(|x| x.iter().map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v)).collect()),
                                transition: transition.map(|x| unsafe { x.as_ref().unwrap() })
                            });
                        }
                        rip += 1;
                    },
                    Behavior::Move(delta) => {
                        if ChessemblyCompiled::is_zero_vector(&delta) {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }

                        let wc = ChessemblyCompiled::move_anchor(&mut stack.last_mut().unwrap().0, &delta, board, board.color_on(position).unwrap());
                        
                        if wc != WallCollision::NoCollision {
                            *states.last_mut().unwrap() = false;
                            rip += 1;
                            continue;
                        }
                        if ChessemblyCompiled::is_friendly(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                            *states.last_mut().unwrap() = false;
                        }
                        else if ChessemblyCompiled::is_enemy(&stack.last().unwrap().0, board, board.color_on(position).unwrap()) {
                            ChessemblyCompiled::cancel_move_anchor(&mut stack.last_mut().unwrap().0, &delta);
                            *states.last_mut().unwrap() = false;
                        }
                        else {
                            ChessemblyCompiled::push_node(&mut nodes, ChessMove {
                                from: position.clone(),
                                take: stack.last_mut().unwrap().0.clone(),
                                move_to: stack.last_mut().unwrap().0.clone(),
                                move_type: MoveType::Move,
                                state_change: state_change.clone().map(|x| x.iter().map(|(k, v)| (unsafe { k.as_ref().unwrap() }, *v)).collect()),
                                transition: transition.map(|x| unsafe { x.as_ref().unwrap() })
                            });
                        }
                        rip += 1;
                    },
                    Behavior::Repeat(n) => {
                        if n == 0 {
                            break;
                        }
                        if n as usize > rip {
                            break;
                        }
                        rip -= n as usize;
                    },
                    Behavior::Not => {
                        let x = *states.last().unwrap();
                        *states.last_mut().unwrap() = !x;
                        rip += 1;
                    },
                    Behavior::Do => {
                        if let Some(next_inst) = chain.get(rip + 1) {
                            match next_inst {
                                Behavior::While => {
                                    rip += 1;
                                },
                                _ => {
                                    states.push(true);
                                }
                            }
                        }
                        else {
                            break;
                        }
                        rip += 1;
                    },
                    Behavior::While => {
                        if *states.last().unwrap() {
                            let mut ss = 0;
                            loop {
                                if chain[rip] == Behavior::While {
                                    ss += 1;
                                }
                                else if chain[rip] == Behavior::Do {
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
                        }
                        else {
                            states.pop();
                            if states.len() == 0 {
                                break;
                            }
                            rip += 1;
                        }
                    },
                    Behavior::Label(_) => {
                        rip += 1;
                    },
                    Behavior::Jmp(label) => {
                        if *states.last().unwrap() {
                            if let Some(label_rip) = chain.iter().enumerate().find(|&(_, v)| *v == Behavior::Label(label)) {
                                rip = label_rip.0;
                            }
                            else {
                                break;
                            }
                        }
                        else {
                            rip += 1;
                            *states.last_mut().unwrap() = true;
                        }
                    },
                    Behavior::Jne(label) => {
                        if !*states.last().unwrap() {
                            if let Some(label_rip) = chain.iter().enumerate().find(|&(_, v)| *v == Behavior::Label(label)) {
                                rip = label_rip.0;
                            }
                            else {
                                break;
                            }
                        }
                        else {
                            rip += 1;
                            *states.last_mut().unwrap() = true;
                        }
                    },
                    _ => break
                };
            }
        }
        return Ok(nodes);
    }

    pub fn filter_nodes(&self, nodes :Vec<ChessMove<'a>>, board :&Board<'a>) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        for testnode in nodes {
            let mut new_board = board.make_move_new_nc(&testnode, false);
            // let turn = new_board.turn.invert();
            // new_board.turn = new_board.turn.invert();
            let turn = new_board.turn.invert();
            // if !self.is_check_dbg(&mut new_board, turn) {
            if !self.is_check(&mut new_board, turn) {
                ret.push(testnode);
            }
            else {
                // worker::console_log!("{:?}", turn);
            }
        }
        
        ret
    }

    pub fn get_moves(&self, board :&mut Board<'a>, position :&Position, check_danger :bool) -> Vec<ChessMove<'a>> {
        if let Some(cached) = board.dp.get(position) {
            return cached.clone();
        }

        let piece_on = board.piece_on(position);
        if let Some(piece) = piece_on {
            // worker::console_log!("{}", piece);
            if piece == "pawn" {
                let ret = self.generate_pawn_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "king" {
                let danger_zones = if check_danger { MoveGen::get_danger_zones(board, board.color_on(position).unwrap().invert()) } else { Vec::new() };
                let ret = self.generate_king_moves(board, position, &danger_zones);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "rook" {
                let ret = self.generate_rook_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "knight" {
                let ret = self.generate_knight_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "bishop" {
                let ret = self.generate_bishop_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "queen" {
                let ret = self.generate_queen_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "tempest-rook" {
                let ret = self.generate_tempest_rook_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "bouncing-bishop" {
                let ret = self.generate_bouncing_bishop_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "dozer" {
                let ret = self.generate_dozer_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "alfil" {
                let ret = self.generate_alfil_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "bard" {
                let ret = self.generate_bard_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "zebra" {
                let ret = self.generate_ij_moves(board, position, 3, 2);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "giraffe" {
                let ret = self.generate_ij_moves(board, position, 4, 1);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "camel" {
                let ret = self.generate_ij_moves(board, position, 3, 1);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else if piece == "cannon" {
                let ret = self.generate_cannon_moves(board, position);
                board.dp.insert((position.0, position.1), ret.clone());
                return ret;
            }
            else {
                let ret = self.generate_moves(board, position, check_danger);
                board.dp.insert((position.0, position.1), ret.clone().unwrap_or(Vec::new()));
                return ret.unwrap_or(Vec::new());
            }
        }
        Vec::new()
    }
}