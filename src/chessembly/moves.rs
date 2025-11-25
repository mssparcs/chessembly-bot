use crate::chessembly::{Behavior, Board, ChessMove, Color, MoveType, Position, WallCollision};
use super::ChessemblyCompiled;


impl<'a> ChessemblyCompiled<'a> {
    pub fn generate_pawn_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        let color = board.color_on(position).unwrap();
        let rank = if color == Color::White { 6 } else { 1 };
        let step1 = if color == Color::White { position.1 - 1 } else { position.1 + 1 };
        let promotion = if color == Color::White { 1 } else { 6 };

        if board.color_on(&(position.0, step1)) == None {
            if position.1 == promotion {
                ret.push(ChessMove {
                    from: position.clone(), take: (position.0, step1), move_to: (position.0, step1), move_type: MoveType::Move, state_change: None,
                    transition: Some("knight")
                });
                ret.push(ChessMove {
                    from: position.clone(), take: (position.0, step1), move_to: (position.0, step1), move_type: MoveType::Move, state_change: None,
                    transition: Some("bishop")
                });
                ret.push(ChessMove {
                    from: position.clone(), take: (position.0, step1), move_to: (position.0, step1), move_type: MoveType::Move, state_change: None,
                    transition: Some("rook")
                });
                ret.push(ChessMove {
                    from: position.clone(), take: (position.0, step1), move_to: (position.0, step1), move_type: MoveType::Move, state_change: None,
                    transition: Some("queen")
                });
            }
            else {
                ret.push(ChessMove { from: position.clone(), take: (position.0, step1), move_to: (position.0, step1), move_type: MoveType::Move, state_change: None, transition: None });
            }
            if position.1 == rank {
                let step2 = if color == Color::White { 4 } else { 3 };
                if board.color_on(&(position.0, step2)) == None {
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0, step2),
                        move_to: (position.0, step2),
                        move_type: MoveType::Move,
                        state_change: Some(vec![("enpassant", 1 as u8)]),
                        transition: None
                    });
                }
            }
        }
        
        if position.0 > 0 {
            if board.color_on(&(position.0 - 1, step1)) == board.color_on(position).map(|x| x.invert()) {
                if position.1 == promotion {
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 - 1, step1), move_to: (position.0 - 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("knight")
                    });
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 - 1, step1), move_to: (position.0 - 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("bishop")
                    });
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 - 1, step1), move_to: (position.0 - 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("rook")
                    });
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 - 1, step1), move_to: (position.0 - 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("queen")
                    });
                }
                else {
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 - 1, step1), move_to: (position.0 - 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: None
                    });
                }
            }
        }
        if position.0 < board.get_width() as u8 - 1 {
            if board.color_on(&(position.0 + 1, step1)) == board.color_on(position).map(|x| x.invert()) {
                if position.1 == promotion {
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 + 1, step1), move_to: (position.0 + 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("knight")
                    });
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 + 1, step1), move_to: (position.0 + 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("bishop")
                    });
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 + 1, step1), move_to: (position.0 + 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("rook")
                    });
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 + 1, step1), move_to: (position.0 + 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: Some("queen")
                    });
                }
                else {
                    ret.push(ChessMove {
                        from: position.clone(), take: (position.0 + 1, step1), move_to: (position.0 + 1, step1), move_type: MoveType::Take, state_change: None,
                        transition: None
                    });
                }
            }
        }

        ret
    }

    pub fn generate_king_moves(&self, board :&mut Board<'a>, position :&Position, danger_zones :&Vec<Position>) -> Vec<ChessMove<'a>> {
        let state_transition = vec![
            ("castling-oo", 0),
            ("castling-ooo", 0)
        ];
        let mut ret = Vec::new();

        for i in (-1 as i8)..2 {
            for j in (-1 as i8)..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                if ChessemblyCompiled::wall_collision(position, &(i, j), board, board.color_on(position).unwrap()) == WallCollision::NoCollision {
                    if board.color_on(&((position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8)) != board.color_on(position) {
                        if !danger_zones.iter().any(|&x| x == ((position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8)) {
                            ret.push(ChessMove {
                                from: position.clone(),
                                take: ((position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8),
                                move_to: ((position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8),
                                move_type: MoveType::TakeMove,
                                state_change: Some(state_transition.clone()),
                                transition: None
                            });
                        }
                    }
                }
            }
        }

        // 캐슬링은 나중에

        ret
    }

    pub fn generate_bishop_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((1, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((1, -1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, -1)), Behavior::Repeat(1)],
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_rook_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((0, -1)), Behavior::Repeat(1)],
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_knight_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((2, 1))],
                vec![Behavior::TakeMove((-2, 1))],
                vec![Behavior::TakeMove((2, -1))],
                vec![Behavior::TakeMove((-2, -1))],
                vec![Behavior::TakeMove((1, 2))],
                vec![Behavior::TakeMove((-1, 2))],
                vec![Behavior::TakeMove((1, -2))],
                vec![Behavior::TakeMove((-1, -2))],
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_queen_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((0, -1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((1, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((1, -1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, -1)), Behavior::Repeat(1)],
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_dozer_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((-2, 1))],
                vec![Behavior::TakeMove((-1, 1))],
                vec![Behavior::TakeMove((0, 1))],
                vec![Behavior::TakeMove((1, 1))],
                vec![Behavior::TakeMove((2, 1))]
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_bouncing_bishop_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        let fs = ChessemblyCompiled::from_script("do take-move(1, 1) while peek(0, 0) edge-right(1, 1) jne(0) take-move(-1, 1) repeat(1) label(0) edge-top(1, 1) jne(1) take-move(1, -1) repeat(1) label(1);do take-move(-1, 1) while peek(0, 0) edge-left(-1, 1) jne(0) take-move(1, 1) repeat(1) label(0) edge-top(-1, 1) jne(1) take-move(-1, -1) repeat(1) label(1);do take-move(1, -1) while peek(0, 0) edge-right(1, -1) jne(0) take-move(-1, -1) repeat(1) label(0) edge-bottom(1, -1) jne(1) take-move(1, 1) repeat(1) label(1);do take-move(-1, -1) while peek(0, 0) edge-left(-1, -1) jne(0) take-move(1, -1) repeat(1) label(0) edge-bottom(-1, -1) jne(1) take-move(-1, 1) repeat(1) label(1);").unwrap();
        let ret = fs.generate_moves(board, position, false).unwrap();
        ret
    }

    pub fn generate_alfil_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((2, 2))],
                vec![Behavior::TakeMove((-2, 2))],
                vec![Behavior::TakeMove((2, -2))],
                vec![Behavior::TakeMove((-2, -2))]
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_ij_moves(&self, board :&mut Board<'a>, position :&Position, i :i8, j :i8) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((i, j))],
                vec![Behavior::TakeMove((-i, j))],
                vec![Behavior::TakeMove((i, -j))],
                vec![Behavior::TakeMove((-i, -j))],
                vec![Behavior::TakeMove((j, i))],
                vec![Behavior::TakeMove((-j, i))],
                vec![Behavior::TakeMove((j, -i))],
                vec![Behavior::TakeMove((-j, -i))],
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_bard_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((2, 2))],
                vec![Behavior::TakeMove((-2, 2))],
                vec![Behavior::TakeMove((2, -2))],
                vec![Behavior::TakeMove((-2, -2))],
                vec![Behavior::TakeMove((2, 0))],
                vec![Behavior::TakeMove((-2, 0))],
                vec![Behavior::TakeMove((0, 2))],
                vec![Behavior::TakeMove((0, -2))]
            ]
        }.generate_moves(board, position, false).unwrap()
    }

// piece(cannon) do take(1, 0) enemy(0, 0) not while jump(1, 0) repeat(1);
// piece(cannon) do take(-1, 0) enemy(0, 0) not while jump(-1, 0) repeat(1);
// piece(cannon) do take(0, 1) enemy(0, 0) not while jump(0, 1) repeat(1);
// piece(cannon) do take(0, -1) enemy(0, 0) not while jump(0, -1) repeat(1);
    pub fn generate_cannon_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::Do, Behavior::Take((1, 0)), Behavior::Enemy((0, 0)), Behavior::Not, Behavior::While, Behavior::Jump((1, 0)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Take((-1, 0)), Behavior::Enemy((0, 0)), Behavior::Not, Behavior::While, Behavior::Jump((-1, 0)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Take((0, 1)), Behavior::Enemy((0, 0)), Behavior::Not, Behavior::While, Behavior::Jump((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Take((0, -1)), Behavior::Enemy((0, 0)), Behavior::Not, Behavior::While, Behavior::Jump((0, -1)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Peek((1, 0)), Behavior::While, Behavior::Friendly((0, 0)), Behavior::Move((1, 0)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Peek((-1, 0)), Behavior::While, Behavior::Friendly((0, 0)), Behavior::Move((-1, 0)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Peek((0, 1)), Behavior::While, Behavior::Friendly((0, 0)), Behavior::Move((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::Do, Behavior::Peek((0, -1)), Behavior::While, Behavior::Friendly((0, 0)), Behavior::Move((0, -1)), Behavior::Repeat(1)]
            ]
        }.generate_moves(board, position, false).unwrap()
    }

    pub fn generate_tempest_rook_moves(&self, board :&mut Board<'a>, position :&Position) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((1, 1)), Behavior::TakeMove((1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((1, 1)), Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((1, -1)), Behavior::TakeMove((1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((1, -1)), Behavior::TakeMove((0, -1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, 1)), Behavior::TakeMove((-1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, 1)), Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, -1)), Behavior::TakeMove((-1, 0)), Behavior::Repeat(1)],
                vec![Behavior::TakeMove((-1, -1)), Behavior::TakeMove((0, -1)), Behavior::Repeat(1)],
            ]
        }.generate_moves(board, position, false).unwrap()
    }
}