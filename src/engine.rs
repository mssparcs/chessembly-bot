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
    impl<'a, const MACHO: bool, const IMPRISONED: bool> GameState for Board<'a, MACHO, IMPRISONED, 8> {
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
                    BoardStatus::Checkmate => {
                        -1_000_000
                    },
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

                        // 센터 근접 가중치: 기물 가치가 낮을수록 중앙에 있을 때 더 높은 보너스
                        let dist = (i as i32 * 2 - 7).abs() + (j as i32 * 2 - 7).abs();
                        let center_bonus = (14 - dist) / value.max(1);

                        if self.color_on(&(i, j)) == Some(Color::White) {
                            score += value + center_bonus;
                        } else {
                            score -= value + center_bonus;
                        }
                    }
                }
            }

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

            // 3. 센터 근접 가중치: 기물 가치가 낮을수록 중앙 접근 시 더 높은 보너스
            // 점수 = (이전 거리 - 이동 후 거리) / 기물 가치
            {
                let src = m.get_source();
                let dst = m.get_dest();
                // 중앙(3.5, 3.5)까지의 맨해튼 거리 근사: |2x - 7| + |2y - 7|
                let src_dist = (src.0 as i32 * 2 - 7).abs() + (src.1 as i32 * 2 - 7).abs();
                let dst_dist = (dst.0 as i32 * 2 - 7).abs() + (dst.1 as i32 * 2 - 7).abs();
                let piece_val = get_piece_value(self.piece_on(&m.get_source()).unwrap_or("pawn")).max(1);
                score += (src_dist - dst_dist) / piece_val;
            }

            // 4. TODO (고급): 나중에는 'Killer Moves' (이전 컷오프를 유발한 조용한 수)
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
    }
}

// -----------------------------------------------------------------------------
// 모듈 2: 알파-베타 검색 (네가맥스 구현)
// -----------------------------------------------------------------------------
pub mod search {
    use rand::seq::SliceRandom;

    use super::game_logic::GameState;

    pub fn find_best_move<S: GameState>(state: &mut S, depth: u8, beam_width: Option<usize>) -> Result<(S::Move, i32), usize> {
        if state.is_terminal() {
            return Err(260);
        }

        let mut best_move = None;
        let mut best_score = -i32::MAX;
        let mut alpha = -i32::MAX;
        let beta = i32::MAX;

        let mut moves = state.get_legal_moves();
        let n = moves.len();
        
        let mut rng = rand::rng();
        moves.shuffle(&mut rng);

        moves.sort_by(|a, b| state.score_move(b).cmp(&state.score_move(a)));
        if let Some(n) = beam_width {
            moves.truncate(n);
        }

        for m in moves {
            // 정렬된 리스트를 사용합니다.
            let mut new_state = state.make_move(&m);

            let score = -negamax(&mut new_state, depth - 1, 10, -beta, -alpha, beam_width);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
            alpha = alpha.max(best_score);
        }

        best_move.map(|m| (m, best_score)).ok_or(n)
    }

    fn negamax<S: GameState>(state: &mut S, depth: u8, hard_depth: u8, mut alpha: i32, beta: i32, beam_width: Option<usize>) -> i32 {
        if depth == 0 || hard_depth == 0 || state.is_terminal() {
            return state.evaluate();
        }

        let damper = 0.9;

        let mut value = -i32::MAX;

        // --- (수 정렬 추가) ---
        // 루트 노드(find_best_move)뿐만 아니라 모든 자식 노드에서도
        // 수 정렬을 수행해야 합니다.
        let mut moves: Vec<_> = state.get_legal_moves().into_iter().map(|node| (state.score_move(&node), node)).collect();
        moves.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        if let Some(n) = beam_width {
            moves.truncate(n);
        }
        // --- (끝) ---
        
        let mut i = 0;
        let mut next_depth = if moves.len() < 4 {
            depth
        } else {
            depth - 1
        };

        for (_, m) in moves {
            // 정렬된 리스트를 사용합니다.
            let mut new_state = state.make_move(&m);
            let score = -negamax(&mut new_state, next_depth, hard_depth - 1, -beta, -alpha, beam_width);
            value = value.max(score);
            alpha = alpha.max(value);
            
            if i < 4 {
                i += 1;
                if i == 4 {
                    next_depth = depth - 1;
                }
            }
            if alpha >= beta {
                // (이것이 킬러 수(Killer Move)가 됩니다.
                // TODO: 'm'을 킬러 수 테이블에 저장)
                break; // Beta Cut-off
            }
        }

        (value as f32 * damper) as i32
    }
}
