use std::cmp::Ordering;

use super::ChessemblyCompiled;
use crate::chessembly::{
    Behavior, ChessMove, Color, DeltaPosition, MoveType, Position, WallCollision, board::Board
};

impl<'a> ChessemblyCompiled<'a> {
    pub fn generate_pawn_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        let color = board.color_on(position).unwrap();
        let rank = if color == Color::White { 6 } else { 1 };
        let step1 = if color == Color::White {
            position.1 - 1
        } else {
            position.1 + 1
        };
        let promotion = if color == Color::White { 1 } else { 6 };

        if board.color_on(&(position.0, step1)) == None {
            if position.1 == promotion {
                ret.push(ChessMove {
                    from: position.clone(),
                    take: (position.0, step1),
                    move_to: (position.0, step1),
                    move_type: MoveType::Move,
                    state_change: None,
                    transition: Some("knight"),
                });
                ret.push(ChessMove {
                    from: position.clone(),
                    take: (position.0, step1),
                    move_to: (position.0, step1),
                    move_type: MoveType::Move,
                    state_change: None,
                    transition: Some("bishop"),
                });
                ret.push(ChessMove {
                    from: position.clone(),
                    take: (position.0, step1),
                    move_to: (position.0, step1),
                    move_type: MoveType::Move,
                    state_change: None,
                    transition: Some("rook"),
                });
                ret.push(ChessMove {
                    from: position.clone(),
                    take: (position.0, step1),
                    move_to: (position.0, step1),
                    move_type: MoveType::Move,
                    state_change: None,
                    transition: Some("queen"),
                });
            } else {
                ret.push(ChessMove {
                    from: position.clone(),
                    take: (position.0, step1),
                    move_to: (position.0, step1),
                    move_type: MoveType::Move,
                    state_change: None,
                    transition: None,
                });
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
                        transition: None,
                    });
                }
            }
        }

        if position.1 == match color {
            Color::White => 3,
            Color::Black => 4
        } {
            let board_state = match color {
                Color::White => &board.board_state.white,
                Color::Black => &board.board_state.black
            };
            if position.0 > 0 {
                if board_state.enpassant.contains(&(position.0 - 1, position.1)) {
                    ret.push(ChessMove {
                        from: position.clone(),
                        move_to: (position.0 - 1, step1),
                        take: (position.0 - 1, position.1),
                        move_type: MoveType::TakeJump,
                        state_change: None,
                        transition: None
                    });
                }
            }
            if position.0 < 7 {
                if board_state.enpassant.contains(&(position.0 + 1, position.1)) {
                    ret.push(ChessMove {
                        from: position.clone(),
                        move_to: (position.0 + 1, step1),
                        take: (position.0 + 1, position.1),
                        move_type: MoveType::TakeJump,
                        state_change: None,
                        transition: None
                    });
                }
            }
        }

        if position.0 > 0 {
            if board.color_on(&(position.0 - 1, step1)) == Some(color.invert()) {
                if position.1 == promotion {
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 - 1, step1),
                        move_to: (position.0 - 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("knight"),
                    });
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 - 1, step1),
                        move_to: (position.0 - 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("bishop"),
                    });
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 - 1, step1),
                        move_to: (position.0 - 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("rook"),
                    });
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 - 1, step1),
                        move_to: (position.0 - 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("queen"),
                    });
                } else {
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 - 1, step1),
                        move_to: (position.0 - 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: None,
                    });
                }
            }
        }
        if position.0 < board.get_width() as u8 - 1 {
            if board.color_on(&(position.0 + 1, step1)) == Some(color.invert()) {
                if position.1 == promotion {
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 + 1, step1),
                        move_to: (position.0 + 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("knight"),
                    });
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 + 1, step1),
                        move_to: (position.0 + 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("bishop"),
                    });
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 + 1, step1),
                        move_to: (position.0 + 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("rook"),
                    });
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 + 1, step1),
                        move_to: (position.0 + 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: Some("queen"),
                    });
                } else {
                    ret.push(ChessMove {
                        from: position.clone(),
                        take: (position.0 + 1, step1),
                        move_to: (position.0 + 1, step1),
                        move_type: MoveType::Take,
                        state_change: None,
                        transition: None,
                    });
                }
            }
        }

        ret
    }

    pub fn generate_king_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
        danger_zones: u64,
    ) -> Vec<ChessMove<'a>> {
        let state_transition = vec![("castling-oo", 0), ("castling-ooo", 0)];
        let mut ret = Vec::new();

        for i in (-1 as i8)..2 {
            for j in (-1 as i8)..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                if ChessemblyCompiled::wall_collision(
                    position,
                    &(i, j),
                    board,
                    board.color_on(position).unwrap(),
                ) == WallCollision::NoCollision
                {
                    if board.color_on(&((position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8))
                        != board.color_on(position)
                    {
                        if !ChessemblyCompiled::is_danger_bit(danger_zones, (position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8) {
                            ret.push(ChessMove {
                                from: position.clone(),
                                take: ((position.0 as i8 + i) as u8, (position.1 as i8 - j) as u8),
                                move_to: (
                                    (position.0 as i8 + i) as u8,
                                    (position.1 as i8 - j) as u8,
                                ),
                                move_type: MoveType::TakeMove,
                                state_change: Some(state_transition.clone()),
                                transition: None,
                            });
                        }
                    }
                }
            }
        }

        let color = board.color_on(position).unwrap();

        let castling_oo = if color == Color::White { board.board_state.white.castling_oo } else { board.board_state.black.castling_oo };
        let castling_ooo = if color == Color::White { board.board_state.white.castling_ooo } else { board.board_state.black.castling_ooo };
        if castling_oo {
            if board.piece_on(&(7, position.1)) == Some("rook") && board.color_on(&(7, position.1)) == Some(color) {
                if board.color_on(&(6, position.1)) == None && board.color_on(&(5, position.1)) == None {
                    if !ChessemblyCompiled::is_danger_bit(danger_zones, position.0, position.1) {
                        ret.push(ChessMove {
                            from: position.clone(),
                            take: (6, position.1),
                            move_to: (6, position.1),
                            move_type: MoveType::Castling,
                            state_change: Some(state_transition.clone()),
                            transition: None,
                        });
                    }
                }
            }
        }
        if castling_ooo {
            if board.piece_on(&(0, position.1)) == Some("rook") && board.color_on(&(0, position.1)) == Some(color) {
                if board.color_on(&(1, position.1)) == None && board.color_on(&(2, position.1)) == None && board.color_on(&(3, position.1)) == None {
                    if !ChessemblyCompiled::is_danger_bit(danger_zones, position.0, position.1) {
                        ret.push(ChessMove {
                            from: position.clone(),
                            take: (2, position.1),
                            move_to: (2, position.1),
                            move_type: MoveType::Castling,
                            state_change: Some(state_transition),
                            transition: None,
                        });
                    }
                }
            }
        }

        ret
    }
    
    pub fn generate_ij_abs_take_move(
        &self,
        moves: &mut Vec<ChessMove<'a>>,
        board: &mut Board<'a>,
        position: &Position,
        delta: &DeltaPosition,
    ) -> bool {
        let mut anchor = position.clone();
        let color = board.color_on(position).unwrap();
        let wc = ChessemblyCompiled::move_anchor(&mut anchor, delta, board, color);
        if wc == WallCollision::NoCollision {
            let color_on = board.color_on(&anchor);
            if color_on == Some(color) {
                false
            }
            else if color_on == None {
                moves.push(ChessMove {
                    from: *position,
                    take: anchor,
                    move_to: anchor,
                    move_type: MoveType::TakeMove,
                    state_change: None,
                    transition: None
                });
                true
            }
            else {
                moves.push(ChessMove {
                    from: *position,
                    take: anchor,
                    move_to: anchor,
                    move_type: MoveType::TakeMove,
                    state_change: None,
                    transition: None
                });
                false
            }
        }
        else {
            false
        }
    }

    pub fn generate_ij_abs_take_move_slide(
        &self,
        moves: &mut Vec<ChessMove<'a>>,
        board: &mut Board<'a>,
        position: &Position,
        delta: &DeltaPosition,
    ) {
        let mut sliding_delta = delta.clone();
        while self.generate_ij_abs_take_move(moves, board, position, &sliding_delta) {
            sliding_delta.0 += delta.0;
            sliding_delta.1 += delta.1;
        }
    }

    pub fn generate_bishop_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = Vec::new();
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, -1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, -1));
        moves
    }

    pub fn generate_rook_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let state_change = match (position.0.cmp(&0), position.0.cmp(&7), position.1.cmp(&0), position.1.cmp(&7), board.color_on(position).unwrap()) {
            (Ordering::Equal, _, _, Ordering::Equal, Color::White) => Some(("castling-ooo", 0)),
            (_, Ordering::Equal, _, Ordering::Equal, Color::White) => Some(("castling-oo", 0)),
            (Ordering::Equal, _, Ordering::Equal, _, Color::Black) => Some(("castling-ooo", 0)),
            (_, Ordering::Equal, Ordering::Equal, _, Color::Black) => Some(("castling-oo", 0)),
            (_, _, _, _, _) => None,
        };
        if let Some(state_transition) = state_change {
            ChessemblyCompiled {
                chains: vec![
                    vec![Behavior::SetState(state_transition), Behavior::TakeMove((1, 0)), Behavior::Repeat(1)],
                    vec![Behavior::SetState(state_transition), Behavior::TakeMove((-1, 0)), Behavior::Repeat(1)],
                    vec![Behavior::SetState(state_transition), Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                    vec![Behavior::SetState(state_transition), Behavior::TakeMove((0, -1)), Behavior::Repeat(1)],
                ],
            }.generate_moves(board, position, false).unwrap()
        }
        else {
            ChessemblyCompiled {
                chains: vec![
                    vec![Behavior::TakeMove((1, 0)), Behavior::Repeat(1)],
                    vec![Behavior::TakeMove((-1, 0)), Behavior::Repeat(1)],
                    vec![Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                    vec![Behavior::TakeMove((0, -1)), Behavior::Repeat(1)],
                ],
            }.generate_moves(board, position, false).unwrap()
        }
    }

    pub fn generate_knight_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        self.generate_ij_moves(board, position, 2, 1)
    }

    pub fn generate_queen_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = Vec::new();
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, -1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, -1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, -1));
        moves
    }

    pub fn generate_dozer_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = Vec::new();
        if board.color_on(position) == Some(Color::Black) {
            self.generate_ij_abs_take_move(&mut moves, board, position, &(-2, -1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(-1, -1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(0, -1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(1, -1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(2, -1));
        }
        else {
            self.generate_ij_abs_take_move(&mut moves, board, position, &(-2, 1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(-1, 1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(0, 1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(1, 1));
            self.generate_ij_abs_take_move(&mut moves, board, position, &(2, 1));
        }
        moves
    }

    pub fn generate_bouncing_bishop_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let fs = ChessemblyCompiled::from_script("do take-move(1, 1) while peek(0, 0) edge-right(1, 1) jne(0) take-move(-1, 1) repeat(1) label(0) edge-top(1, 1) jne(1) take-move(1, -1) repeat(1) label(1);do take-move(-1, 1) while peek(0, 0) edge-left(-1, 1) jne(0) take-move(1, 1) repeat(1) label(0) edge-top(-1, 1) jne(1) take-move(-1, -1) repeat(1) label(1);do take-move(1, -1) while peek(0, 0) edge-right(1, -1) jne(0) take-move(-1, -1) repeat(1) label(0) edge-bottom(1, -1) jne(1) take-move(1, 1) repeat(1) label(1);do take-move(-1, -1) while peek(0, 0) edge-left(-1, -1) jne(0) take-move(1, -1) repeat(1) label(0) edge-bottom(-1, -1) jne(1) take-move(-1, 1) repeat(1) label(1);").unwrap();
        let ret = fs.generate_moves(board, position, false).unwrap();
        ret
    }

    pub fn generate_alfil_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        self.generate_ij_abs_take_move(&mut ret, board, position, &(2, 2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(2, -2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-2, 2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-2, -2));
        ret
    }

    pub fn generate_ij_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
        i: i8,
        j: i8,
    ) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        self.generate_ij_abs_take_move(&mut ret, board, position, &(i, j));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-i, j));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(i, -j));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-i, -j));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(j, i));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-j, i));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(j, -i));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-j, -i));
        ret
    }

    pub fn generate_bard_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut ret = Vec::new();
        self.generate_ij_abs_take_move(&mut ret, board, position, &(2, 0));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-2, 0));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(0, 2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(0, -2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(2, 2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(2, -2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-2, 2));
        self.generate_ij_abs_take_move(&mut ret, board, position, &(-2, -2));
        ret
    }
    
    pub fn generate_wasp_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::TakeMove((0, 1)), Behavior::Repeat(1)],
                vec![Behavior::Move((1, -1)), Behavior::Repeat(1)],
                vec![Behavior::Move((-1, -1)), Behavior::Repeat(1)],
            ],
        }
            .generate_moves(board, position, false)
            .unwrap()
    }

    pub fn generate_amazon_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = self.generate_knight_moves(board, position);
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, -1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, -1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, -1));
        moves
    }

    pub fn generate_centaur_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = self.generate_knight_moves(board, position);
        self.generate_ij_abs_take_move(&mut moves, board, position, &(1, 0));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(-1, 0));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(0, 1));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(0, -1));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(1, 1));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(1, -1));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(-1, 1));
        self.generate_ij_abs_take_move(&mut moves, board, position, &(-1, -1));
        moves
    }

    pub fn generate_archbishop_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = self.generate_knight_moves(board, position);
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, -1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, -1));
        moves
    }

    pub fn generate_chancellor_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = self.generate_knight_moves(board, position);
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, -1));
        moves
    }

    pub fn generate_cannon_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![
                    Behavior::Do,
                    Behavior::Take((1, 0)),
                    Behavior::Enemy((0, 0)),
                    Behavior::Not,
                    Behavior::While,
                    Behavior::Jump((1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Take((-1, 0)),
                    Behavior::Enemy((0, 0)),
                    Behavior::Not,
                    Behavior::While,
                    Behavior::Jump((-1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Take((0, 1)),
                    Behavior::Enemy((0, 0)),
                    Behavior::Not,
                    Behavior::While,
                    Behavior::Jump((0, 1)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Take((0, -1)),
                    Behavior::Enemy((0, 0)),
                    Behavior::Not,
                    Behavior::While,
                    Behavior::Jump((0, -1)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Peek((1, 0)),
                    Behavior::While,
                    Behavior::Friendly((0, 0)),
                    Behavior::Move((1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Peek((-1, 0)),
                    Behavior::While,
                    Behavior::Friendly((0, 0)),
                    Behavior::Move((-1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Peek((0, 1)),
                    Behavior::While,
                    Behavior::Friendly((0, 0)),
                    Behavior::Move((0, 1)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::Do,
                    Behavior::Peek((0, -1)),
                    Behavior::While,
                    Behavior::Friendly((0, 0)),
                    Behavior::Move((0, -1)),
                    Behavior::Repeat(1),
                ],
            ],
        }
            .generate_moves(board, position, false)
            .unwrap()
    }

    pub fn generate_tempest_rook_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![
                    Behavior::TakeMove((1, 1)),
                    Behavior::TakeMove((1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((1, 1)),
                    Behavior::TakeMove((0, 1)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((1, -1)),
                    Behavior::TakeMove((1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((1, -1)),
                    Behavior::TakeMove((0, -1)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((-1, 1)),
                    Behavior::TakeMove((-1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((-1, 1)),
                    Behavior::TakeMove((0, 1)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((-1, -1)),
                    Behavior::TakeMove((-1, 0)),
                    Behavior::Repeat(1),
                ],
                vec![
                    Behavior::TakeMove((-1, -1)),
                    Behavior::TakeMove((0, -1)),
                    Behavior::Repeat(1),
                ],
            ],
        }
            .generate_moves(board, position, false)
            .unwrap()
    }

    pub fn generate_chameleon_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = ChessemblyCompiled {
            chains: vec![
                vec![Behavior::Move((1, 0))],
                vec![Behavior::Move((-1, 0))],
                vec![Behavior::Move((0, 1))],
                vec![Behavior::Move((0, -1))],
                vec![Behavior::Move((1, 1))],
                vec![Behavior::Move((1, -1))],
                vec![Behavior::Move((-1, 1))],
                vec![Behavior::Move((-1, -1))],
            ],
        }
            .generate_moves(board, position, false)
            .unwrap();

        let catch_list = [(2, 2), (2, -2), (-2, 2), (-2, -2)];
        for catch_delta in catch_list {
            let mut anchor = position.clone();
            let wc = ChessemblyCompiled::move_anchor(&mut anchor, &catch_delta, board, board.color_on(position).unwrap());
            if wc == WallCollision::NoCollision {
                if board.color_on(&anchor) == Some(board.color_on(position).unwrap().invert()) {
                    match board.piece_on(&anchor).unwrap() {
                        "pawn" => {
                            moves.push(ChessMove {
                                from: *position,
                                take: anchor.clone(),
                                move_to: *position,
                                move_type: MoveType::Catch,
                                state_change: None,
                                transition: Some("mirrored-pawn")
                            });
                        },
                        "queen" => {
                            moves.push(ChessMove {
                                from: *position,
                                take: anchor.clone(),
                                move_to: *position,
                                move_type: MoveType::Catch,
                                state_change: None,
                                transition: Some("mirrored-queen")
                            });
                        },
                        "bishop" => {
                            moves.push(ChessMove {
                                from: *position,
                                take: anchor.clone(),
                                move_to: *position,
                                move_type: MoveType::Catch,
                                state_change: None,
                                transition: Some("mirrored-bishop")
                            });
                        },
                        "knight" => {
                            moves.push(ChessMove {
                                from: *position,
                                take: anchor.clone(),
                                move_to: *position,
                                move_type: MoveType::Catch,
                                state_change: None,
                                transition: Some("mirrored-knight")
                            });
                        },
                        "rook" => {
                            moves.push(ChessMove {
                                from: *position,
                                take: anchor.clone(),
                                move_to: *position,
                                move_type: MoveType::Catch,
                                state_change: None,
                                transition: Some("mirrored-rook")
                            });
                        },
                        _ => {}
                    }
                }
            }
        }
        moves
    }

    pub fn generate_pseudo_pawn_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        ChessemblyCompiled {
            chains: vec![
                vec![Behavior::Move((0, 1))],
                vec![Behavior::Take((1, 1))],
                vec![Behavior::Take((-1, 1))],
            ],
        }
            .generate_moves(board, position, false)
            .unwrap()
    }

    pub fn generate_beacon_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let color = board.color_on(position).unwrap();
        let mut moves = Vec::new();
        for i in 0..8 {
            for j in 0..8 {
                let Some(target_color) = board.color_on(&(j, i)) else { continue; };
                if target_color == color {
                    let target_piece = board.piece_on(&(j, i)).unwrap();
                    if target_piece == "pawn" {
                        continue;
                    }
                    else if target_piece == "beacon" {
                        continue;
                    }
                    moves.push(ChessMove {
                        from: *position,
                        take: *position,
                        move_to: (j, i),
                        move_type: MoveType::Shift,
                        state_change: None,
                        transition: None
                    });
                }
            }
        }
        moves
    }

    pub fn generate_pseudo_rook_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let mut moves = Vec::new();
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(-1, 0));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, 1));
        self.generate_ij_abs_take_move_slide(&mut moves, board, position, &(0, -1));
        moves
    }
    
    pub fn generate_windmill_rook_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let fs = ChessemblyCompiled::from_script("piece(windmill-rook) transition(windmill-bishop) { take-move(1, 0) repeat(1) } { take-move(0, 1) repeat(1) } { take-move(-1, 0) repeat(1) } { take-move(0, -1) repeat(1) };").unwrap();
        let ret = fs.generate_moves(board, position, false).unwrap();
        ret
    }

    pub fn generate_windmill_bishop_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
    ) -> Vec<ChessMove<'a>> {
        let fs = ChessemblyCompiled::from_script("piece(windmill-bishop) transition(windmill-rook) { take-move(1, 1) repeat(1) } { take-move(-1, 1) repeat(1) } { take-move(1, -1) repeat(1) } { take-move(-1, -1) repeat(1) };").unwrap();
        let ret = fs.generate_moves(board, position, false).unwrap();
        ret
    }

    pub fn generate_mirrored_moves(
        &self,
        board: &mut Board<'a>,
        position: &Position,
        piece: &str,
    ) -> Vec<ChessMove<'a>> {
        let moves = match piece {
            "mirrored-pawn" => self.generate_pseudo_pawn_moves(board, position),
            "mirrored-bishop" => self.generate_bishop_moves(board, position),
            "mirrored-rook" => self.generate_pseudo_rook_moves(board, position),
            "mirrored-knight" => self.generate_knight_moves(board, position),
            "mirrored-queen" => self.generate_queen_moves(board, position),
            _ => Vec::new()
        };
        let enemy_color = board.color_on(position).unwrap().invert();

        moves.into_iter().map(|node| {
            let take_color = board.color_on(&node.take);
            if take_color == Some(enemy_color) {
                match board.piece_on(&node.take).unwrap() {
                    "pawn" => ChessMove {
                        from: node.from,
                        take: node.take,
                        move_to: node.move_to,
                        move_type: node.move_type,
                        state_change: None,
                        transition: Some("mirrored-pawn")
                    },
                    "bishop" => ChessMove {
                        from: node.from,
                        take: node.take,
                        move_to: node.move_to,
                        move_type: node.move_type,
                        state_change: None,
                        transition: Some("mirrored-bishop")
                    },
                    "rook" => ChessMove {
                        from: node.from,
                        take: node.take,
                        move_to: node.move_to,
                        move_type: node.move_type,
                        state_change: None,
                        transition: Some("mirrored-rook")
                    },
                    "knight" => ChessMove {
                        from: node.from,
                        take: node.take,
                        move_to: node.move_to,
                        move_type: node.move_type,
                        state_change: None,
                        transition: Some("mirrored-knight")
                    },
                    "queen" => ChessMove {
                        from: node.from,
                        take: node.take,
                        move_to: node.move_to,
                        move_type: node.move_type,
                        state_change: None,
                        transition: Some("mirrored-queen")
                    },
                    _ => node
                }
            }
            else {
                node
            }
        }).collect()
    }
}
