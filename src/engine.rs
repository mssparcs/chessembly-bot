// -----------------------------------------------------------------------------
// 모듈 1: 게임 로직 추상화 (변형 체스를 위한 설계)
// -----------------------------------------------------------------------------
pub mod game_logic {
    use crate::chessembly;
    use chessembly::board::Board;
    use chessembly::board::BoardStatus;
    use chessembly::ChessMove;
    use chessembly::MoveGen;
    use chessembly::Color;
    // use rand::prelude::*; 
    // use crate::chess::{self, Board, ChessMove, Color, GameStatus, MoveGen, Piece};

    /// 모든 게임의 '수'가 구현해야 하는 기본 트레이트.
    /// Debug와 Clone은 검색 트리에 필수적입니다.
    pub trait GameMove: std::fmt::Debug + Clone {}

    /// 'chess' 라이브러리의 ChessMove에 우리 트레이트를 구현.
    impl<'a> GameMove for ChessMove<'a> {}

    /// 모든 게임 상태(보드)가 구현해야 하는 트레이트.
    /// 이 트레이트만 구현하면 어떤 게임이든 우리 검색 알고리즘을 쓸 수 있습니다.
    pub trait GameState: Clone {
        type Move: GameMove;

        fn get_legal_moves(&mut self) -> Vec<Self::Move>;
        fn make_move(&self, m: &Self::Move) -> Self;
        fn is_terminal(&self) -> bool;
        fn evaluate(&mut self) -> i32;

        /// (추가됨) 수 정렬을 위한 휴리스틱 점수 반환
        /// 이 점수는 '평가(evaluate)'와 다릅니다. 이 수는 즉각적으로
        /// 얼마나 "공격적인" 수인지를 나타냅니다. (예: 캡처, 프로모션)
        /// 높을수록 먼저 탐색되어야 합니다.
        fn score_move(&self, m: &Self::Move) -> i32;
    }

    // --- 표준 체스를 위한 GameState 구현 ---
    // 'chess::Board'에 우리가 정의한 GameState 트레이트를 구현합니다.
    // 만약 '변형 체스'를 만드신다면,
    // 'MyVariantBoard' 같은 자신만의 구조체를 만들고 이 트레이트를 구현하면 됩니다.
    impl<'a> GameState for Board<'a> {
        type Move = ChessMove<'a>;

        fn get_legal_moves(&mut self) -> Vec<Self::Move> {
            // MoveGen을 사용해 모든 합법적인 수를 생성합니다.
            MoveGen::new_legal(self)
        }

        fn make_move(&self, m: &Self::Move) -> Self {
            // 'chess' 보드의 'make_move_new'는 수를 적용한 새 보드를 반환합니다.
            self.make_move_new(&m)
        }

        fn is_terminal(&self) -> bool {
            // 게임 상태가 '진행 중'이 아니면 종료된 것입니다.
            self.status() != BoardStatus::Ongoing
        }

        /// 현재 턴인 플레이어의 관점에서 보드 점수를 계산합니다.
        fn evaluate(&mut self) -> i32 {
            // 1. 게임 종료 상태 확인
            if self.is_terminal() {
                return match self.status() {
                    // 현재 플레이어가 체크메이트 당함 (최악의 점수)
                    BoardStatus::Checkmate => -1_000_000,
                    // 무승부
                    BoardStatus::Stalemate => 0,
                    _ => 0,
                };
            }

            // 2. 기물 가치 계산 (단순한 예시)
            let mut score = 0;
            for i in 0..8 {
                for j in 0..8 {
                    if let Some(piece) = self.piece_on(&(i, j)) {
                        let value = get_piece_value(piece);
                        if self.color_on(&(i, j)) == Some(Color::White) {
                            if self.side_to_move() == Color::White {
                                score += value * 5;
                            } else {
                                score += value * 5;
                            }
                        } else {
                            if self.side_to_move() == Color::White {
                                score -= value * 5;
                            } else {
                                score -= value * 5;
                            }
                        }
                    }
                }
            }

            // let dz1 = MoveGen::get_danger_zones(self, Color::White).len();
            // let dz2 = MoveGen::get_danger_zones(self, Color::Black).len();
            // score += dz1 as i32 - dz2 as i32;

            // 3. 현재 턴인 플레이어에 맞춰 점수 반환
            // 백의 턴이면 (백 - 흑) 점수 반환
            // 흑의 턴이면 (흑 - 백) 점수 반환
            if self.side_to_move() == Color::White {
                score
            } else {
                -score
            }
        }

        fn score_move(&self, m: &Self::Move) -> i32 {
            let mut score = 0;

            // 1. 프로모션: 퀸 프로모션이 가장 높은 점수를 가집니다.
            if let Some(promoted_piece) = m.get_promotion() {
                // 기본 1000점에 + 프로모션 기물 가치
                score += 50 + 5 * get_piece_value(promoted_piece);
            }

            // 2. 캡처 (기물 잡기)
            // 'to' 스퀘어에 상대방 기물이 있는지 확인합니다.
            if let Some(victim) = self.piece_on(&m.get_dest()) {
                // 'from' 스퀘어에 있는 내 기물 (공격자)
                // unwrap_or(Pawn)은 캐슬링 같은 특수 경우에도 패닉이 나지 않도록 합니다.
                let attacker = self.piece_on(&m.get_source()).unwrap_or("pawn");

                // MVV-LVA (Most Valuable Victim, Least Valuable Attacker) 휴리스틱
                // (잡힌 기물 가치 * 10) - (공격 기물 가치)
                // 예: 폰으로 퀸 잡기: (900 * 10) - 100 = 8900 점
                // 예: 퀸으로 폰 잡기: (100 * 10) - 900 = 100 점
                // 이렇게 하면 가치 높은 기물을 잡는 수가 압도적으로 높은 우선순위를 갖게 됩니다.
                // score += (get_piece_value(victim) * 10) - get_piece_value(attacker);
                score += get_piece_value(victim) * 50 - get_piece_value(attacker) * 5;
            }

            // 3. TODO (고급): 나중에는 'Killer Moves' (이전 컷오프를 유발한 조용한 수)
            // 4. TODO (고급): 나중에는 'History Heuristic' (과거에 좋았던 수)

            // 캡처나 프로모션이 아닌 '조용한 수(quiet move)'는 0점을 반환합니다.
            
            score
        }
    }

    /// 기물의 가치를 반환하는 헬퍼 함수
    fn get_piece_value(piece: &str) -> i32 {
        if piece == "pawn" {
            return 1;
        } else if piece == "knight" {
            return 3;
        } else if piece == "bishop" {
            return 3;
        } else if piece == "rook" {
            return 5;
        } else if piece == "queen" {
            return 9;
        } else if piece == "king" {
            return 10000;
        } else {
            return 8;
        }

        // match piece {
        //     Piece::Pawn => 1,
        //     Piece::Knight => 3,
        //     Piece::Bishop => 3, // 비숍을 나이트보다 약간 높게 평가
        //     Piece::Rook => 5,
        //     Piece::Queen => 9,
        //     Piece::King => 10000, // 킹은 사실상 무한대 가치지만, 평가에서는 제외
        // }
    }
}

// -----------------------------------------------------------------------------
// 모듈 2: 알파-베타 검색 (네가맥스 구현)
// -----------------------------------------------------------------------------
pub mod search {
    use rand::seq::SliceRandom;

    use super::game_logic::GameState;

    /// 지정된 깊이(depth)까지 탐색하여 최선의 수를 찾습니다.
    /// (S: GameState)는 'GameState' 트레이트를 구현한 어떤 게임이든 받는다는 의미입니다.
    // pub fn find_best_move<S: GameState>(state: &S, depth: u8) -> Option<(S::Move, i32)> {
    //     if state.is_terminal() {
    //         return None;
    //     }

    //     let mut best_move = None;
    //     let mut best_score = -i32::MAX; // 음의 무한대

    //     // 알파-베타 가지치기를 위한 초기값
    //     let mut alpha = -i32::MAX;
    //     let beta = i32::MAX;

    //     // TODO: 수(Move) 정렬을 구현하면 성능이 비약적으로 향상됩니다.
    //     // (예: 잡는 수 먼저 탐색하기)
    //     let moves = state.get_legal_moves();

    //     for m in moves {
    //         let new_state = state.make_move(&m);

    //         // 네가맥스 호출:
    //         // 점수에 -를 붙이고, alpha/beta를 뒤집어(-beta, -alpha) 전달합니다.
    //         let score = -negamax(&new_state, depth - 1, -beta, -alpha);

    //         if score > best_score {
    //             best_score = score;
    //             best_move = Some(m);
    //         }

    //         // 루트 노드에서의 알파값 갱신
    //         alpha = alpha.max(best_score);
    //     }

    //     best_move.map(|m| (m, best_score))
    // }

    /// 네가맥스(Negamax) 알고리즘을 사용한 알파-베타 가지치기 함수
    ///
    /// `alpha`: 현재 플레이어가 보장받을 수 있는 최소 점수 (하한선)
    /// `beta`: 상대방이 허용하는 최대 점수 (상한선)
    ///
    /// 현재 노드의 점수가 `beta`보다 크거나 같으면,
    /// 이 노드의 부모(상대방 턴)는 이 수를 절대 선택하지 않을 것입니다.
    /// (상대방은 이미 `beta`보다 *낮은* 점수를 보장받았으므로)
    /// 따라서 더 이상 탐색할 필요가 없습니다 (Beta Cut-off).
    // fn negamax<S: GameState>(state: &S, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    //     // 1. 깊이 한계에 도달했거나 게임이 종료되었으면, 현재 상태를 평가하고 반환
    //     if depth == 0 || state.is_terminal() {
    //         return state.evaluate();
    //     }

    //     let mut value = -i32::MAX; // 음의 무한대

    //     for m in state.get_legal_moves() {
    //         let new_state = state.make_move(&m);

    //         // 2. 재귀 호출 (상대방의 점수는 나에게 -가 됨)
    //         let score = -negamax(&new_state, depth - 1, -beta, -alpha);

    //         // 3. 더 나은 점수를 갱신
    //         value = value.max(score);

    //         // 4. Alpha값 갱신 (내가 보장받을 수 있는 최소 점수)
    //         alpha = alpha.max(value);

    //         // 5. 알파-베타 컷오프 (Beta Cut-off)
    //         //    내가 찾은 점수(alpha)가 상대방의 상한선(beta)보다 크거나 같으면,
    //         //    상대방은 이쪽 분기(branch)를 절대 선택하지 않으므로 탐색 중단.
    //         if alpha >= beta {
    //             break;
    //         }
    //     }

    //     value
    // }

    // ... in mod search

    pub fn find_best_move<S: GameState>(state: &mut S, depth: u8) -> Result<(S::Move, i32), usize> {
        if state.is_terminal() {
            return Err(260);
        }

        let mut best_move = None;
        let mut best_score = -i32::MAX;
        let mut alpha = -i32::MAX;
        let beta = i32::MAX;

        let mut moves = state.get_legal_moves();
        let n = moves.len();

        // --- (수 정렬 추가) ---
        // score_move 점수가 높은 순 (내림차순)으로 정렬합니다.
        // b가 a보다 앞에 오도록 비교합니다. (unstable_by가 더 빠름)

        let mut rng = rand::rng();
        moves.shuffle(&mut rng);

        // let len = moves.len();
        // for i in (1..len).rev() {
            
        //     let j = unsafe { (worker::js_sys::Math::random() * len as f64).to_int_unchecked() };
        //     moves.swap(i, j);
        // }

        moves.sort_by(|a, b| state.score_move(b).cmp(&state.score_move(a)));
        // --- (끝) ---

        for m in moves {
            // 정렬된 리스트를 사용합니다.
            let mut new_state = state.make_move(&m);

            let score = -negamax(&mut new_state, depth - 1, -beta, -alpha);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
            alpha = alpha.max(best_score);
        }

        best_move.map(|m| (m, best_score)).ok_or(n)
    }

    fn negamax<S: GameState>(state: &mut S, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 || state.is_terminal() {
            return state.evaluate();
        }

        let mut value = -i32::MAX;

        // --- (수 정렬 추가) ---
        // 루트 노드(find_best_move)뿐만 아니라 모든 자식 노드에서도
        // 수 정렬을 수행해야 합니다.
        let mut moves = state.get_legal_moves();
        moves.sort_unstable_by(|a, b| state.score_move(b).cmp(&state.score_move(a)));
        // --- (끝) ---

        for m in moves {
            // 정렬된 리스트를 사용합니다.
            let mut new_state = state.make_move(&m);
            let score = -negamax(&mut new_state, depth - 1, -beta, -alpha);
            value = value.max(score);
            alpha = alpha.max(value);
            if alpha >= beta {
                // (이것이 킬러 수(Killer Move)가 됩니다.
                // TODO: 'm'을 킬러 수 테이블에 저장)
                break; // Beta Cut-off
            }
        }

        // worker::console_log!("{}", score);

        value
    }
}

// -----------------------------------------------------------------------------
// 메인 실행 함수
// -----------------------------------------------------------------------------
// pub fn search_best(board :&Board) {
//     // FEN 표기법을 사용해 특정 보드 상태에서 시작할 수도 있습니다.
//     // let board = Board::from_str("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3").unwrap();

//     // 기본 시작 위치
//     // let board = Board::from_str("r1bqkbnr/ppp1p1pp/2np1p2/4Q3/8/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1").unwrap();
//     // let compiled = ChessemblyCompiled::from_script("do take-move(1, -1) while peek(0, 0) edge-right(1, -1) jne(0) take-move(-1, -1) repeat(1) label(0) edge-bottom(1, -1) jne(1) take-move(1, 1) repeat(1) label(1);do take-move(1, -1) while peek(0, 0) edge-right(1, -1) jne(0) take-move(-1, -1) repeat(1) label(0) edge-bottom(1, -1) jne(1) take-move(1, 1) repeat(1) label(1);").unwrap();
//     // let mut board = Board::from_str("r bqkbnr/ppp p pp/  np p  /    Q   /        /        /PPPPPPPP/RNB KBNR/", &compiled);
//     // board.turn = Color::Black;

//     let depth = 4; // 탐색 깊이 (값이 클수록 더 오래 걸리고 더 강해집니다)

//     // println!("시작 보드 상태:\n{:?}", board);
//     // println!("탐색 깊이: {}", depth);

//     match search::find_best_move(board, depth) {
//         Some((best_move, score)) => {
//             // println!("\n--- 결과 ---");
//             // println!("찾은 최선의 수: {:?}", best_move);
//             println!("예상 점수: {}", score);

//             let final_board = board.make_move_new(&best_move);
//             println!("\n적용 후 보드 상태:\n{}", final_board.to_string());
//         }
//         None => {
//             println!("게임이 이미 종료되었거나 수를 찾을 수 없습니다.");
//         }
//     };
// }
