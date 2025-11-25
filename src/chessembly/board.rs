use super::{ChessMove, ChessemblyCompiled, Color, HashMap, MoveGen, Piece, PieceSpan, Position};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Eq)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BoardState<'a> {
    pub castling_oo: bool,
    pub castling_ooo: bool,
    pub enpassant: Vec<Position>,
    pub register: HashMap<&'a str, u8>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BothBoardState<'a> {
    pub black: BoardState<'a>,
    pub white: BoardState<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board<'a> {
    pub board: [[PieceSpan<'a>; 8]; 8],
    pub board_state: BothBoardState<'a>,
    pub turn: Color,
    pub script: &'a ChessemblyCompiled<'a>,
    pub status: BoardStatus,
    pub dp: HashMap<Position, Vec<ChessMove<'a>>>,
}

impl<'a> Board<'a> {
    pub fn from_str(placement: &str, script: &'a ChessemblyCompiled) -> Board<'a> {
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
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
            ],
            board_state: BothBoardState {
                black: BoardState {
                    castling_oo: true,
                    castling_ooo: true,
                    enpassant: Vec::new(),
                    register: HashMap::new(),
                },
                white: BoardState {
                    castling_oo: true,
                    castling_ooo: true,
                    enpassant: Vec::new(),
                    register: HashMap::new(),
                },
            },
            script: script,
            turn: Color::White,
            status: BoardStatus::Ongoing,
        };
        for i in 0..8 {
            for j in 0..8 {
                let Some(char) = placement.chars().nth(i * 9 + j) else {
                    continue;
                };

                let piece = match char {
                    'Q' => ("queen", Color::White),
                    'N' => ("knight", Color::White),
                    'K' => ("king", Color::White),
                    'B' => ("bishop", Color::White),
                    'R' => ("rook", Color::White),
                    'P' => ("pawn", Color::White),

                    'q' => ("queen", Color::Black),
                    'n' => ("knight", Color::Black),
                    'k' => ("king", Color::Black),
                    'b' => ("bishop", Color::Black),
                    'r' => ("rook", Color::Black),
                    'p' => ("pawn", Color::Black),

                    _ => continue,
                };

                ret.board[i][j] = PieceSpan::Piece(Piece {
                    piece_type: piece.0,
                    color: piece.1,
                });
            }
        }
        ret
    }

    pub fn empty(script: &'a ChessemblyCompiled) -> Board<'a> {
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
                [PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty, PieceSpan::Empty],
            ],
            board_state: BothBoardState {
                black: BoardState {
                    castling_oo: true,
                    castling_ooo: true,
                    enpassant: Vec::new(),
                    register: HashMap::new(),
                },
                white: BoardState {
                    castling_oo: true,
                    castling_ooo: true,
                    enpassant: Vec::new(),
                    register: HashMap::new(),
                },
            },
            script,
            turn: Color::White,
            status: BoardStatus::Ongoing,
        }
    }

    pub fn new(script: &'a ChessemblyCompiled) -> Board<'a> {
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
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "rook" }),
                ],
                [
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
                    PieceSpan::Piece(Piece { color: Color::Black, piece_type: "pawn" }),
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
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "pawn" }),
                ],
                [
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "rook" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "knight" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "bishop" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "queen" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "king" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "bishop" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "knight" }),
                    PieceSpan::Piece(Piece { color: Color::White, piece_type: "rook" }),
                ],
            ],
            board_state: BothBoardState {
                black: BoardState {
                    castling_oo: true,
                    castling_ooo: true,
                    enpassant: Vec::new(),
                    register: HashMap::new(),
                },
                white: BoardState {
                    castling_oo: true,
                    castling_ooo: true,
                    enpassant: Vec::new(),
                    register: HashMap::new(),
                },
            },
            script,
            turn: Color::White,
            status: BoardStatus::Ongoing,
        }
    }

    pub fn to_string(&self) -> String {
        let mut ret = String::new();
        for j in 0..8 {
            for i in 0..8 {
                let Some(color) = self.color_on(&[i, j].into()) else {
                    ret.push(' ');
                    continue;
                };
                let piece = self.piece_on(&(i, j)).unwrap();

                let ch = match (piece, color) {
                    ("pawn", Color::Black) => 'p',
                    ("pawn", Color::White) => 'P',
                    ("rook", Color::Black) => 'r',
                    ("rook", Color::White) => 'R',
                    ("bishop", Color::Black) => 'b',
                    ("bishop", Color::White) => 'B',
                    ("knight", Color::Black) => 'n',
                    ("knight", Color::White) => 'N',
                    ("king", Color::Black) => 'k',
                    ("king", Color::White) => 'K',
                    ("queen", Color::Black) => 'q',
                    ("queen", Color::White) => 'Q',
                    _ => {
                        if let Some(piece) = self.piece_on(&(i, j)) {
                            piece.chars().next().unwrap()
                        } else {
                            continue;
                        }
                    }
                };
                ret.push(ch);
            }
            ret.push('\n');
        }
        ret
    }

    pub fn make_move_new_nc(&self, node: &ChessMove<'a>, decide: bool) -> Board<'a> {
        let mut ret = self.clone();
        ret.dp = HashMap::new();
        ret.board[node.take.1 as usize][node.take.0 as usize] = PieceSpan::Empty;
        ret.board[node.move_to.1 as usize][node.move_to.0 as usize] = node
            .transition
            .as_ref()
            .map(|x| {
                PieceSpan::Piece(Piece {
                    piece_type: x,
                    color: match &ret.board[node.from.1 as usize][node.from.0 as usize] {
                        PieceSpan::Empty => Color::White,
                        PieceSpan::Piece(piece) => piece.color,
                    },
                })
            })
            .unwrap_or(ret.board[node.from.1 as usize][node.from.0 as usize].clone());
        ret.board[node.from.1 as usize][node.from.0 as usize] = PieceSpan::Empty;
        if let Some(state_changes) = &node.state_change {
            for (key, n) in state_changes {
                if key == &"castling-oo" {
                    if ret.turn == Color::White {
                        ret.board_state.white.castling_oo = *n > 0;
                    } else if ret.turn == Color::Black {
                        ret.board_state.black.castling_oo = *n > 0;
                    }
                } else if key == &"castling-ooo" {
                    if ret.turn == Color::White {
                        ret.board_state.white.castling_ooo = *n > 0;
                    } else if ret.turn == Color::Black {
                        ret.board_state.black.castling_ooo = *n > 0;
                    }
                } else if key == &"en-passant" {
                    if ret.turn == Color::White {
                        ret.board_state.black.enpassant.push(node.move_to.clone());
                    } else if ret.turn == Color::Black {
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
            } else {
                ret.status = BoardStatus::Stalemate;
            }
        }
        ret
    }

    #[inline]
    pub fn make_move_new(&self, node: &ChessMove<'a>) -> Board<'a> {
        self.make_move_new_nc(node, true)
    }

    #[inline]
    pub const fn status(&self) -> BoardStatus {
        self.status
    }

    #[inline]
    pub const fn piece_on(&self, position: &Position) -> Option<&str> {
        if position.0 > 7 || position.1 > 7 {
            return None;
        } else if let PieceSpan::Piece(piece) =
            &self.board[position.1 as usize][position.0 as usize]
        {
            return Some(&piece.piece_type);
        }
        None
    }

    #[inline]
    pub const fn color_on(&self, position: &Position) -> Option<Color> {
        if position.0 > 7 || position.1 > 7 {
            return None;
        } else if let PieceSpan::Piece(piece) =
            &self.board[position.1 as usize][position.0 as usize]
        {
            return Some(piece.color);
        }
        None
    }

    #[inline]
    pub const fn side_to_move(&self) -> Color {
        self.turn
    }

    #[inline]
    pub const fn get_width(&self) -> usize {
        8
    }

    #[inline]
    pub const fn get_height(&self) -> usize {
        8
    }
}
