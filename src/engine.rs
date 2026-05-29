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
    use crate::engine_huristic::heuristics;

    /// 모든 게임의 '수'가 구현해야 하는 기본 트레이트.
    /// PartialEq는 킬러 테이블 비교에 필요합니다.
    pub trait GameMove: std::fmt::Debug + Clone + PartialEq {
        /// 수의 해시값 반환 (히스토리 휴리스틱용).
        /// 기본값 0을 제공해 기존 구현체가 그대로 컴파일됩니다.
        fn move_hash(&self) -> u64 { 0 }
    }

    /// 'chess' 라이브러리의 ChessMove에 우리 트레이트를 구현.
    impl<'a> GameMove for ChessMove<'a> {
        fn move_hash(&self) -> u64 {
            let src = self.get_source();
            let dst = self.get_dest();
            (src.0 as u64)
                | ((src.1 as u64) << 8)
                | ((dst.0 as u64) << 16)
                | ((dst.1 as u64) << 24)
        }
    }

    /// 모든 게임 상태(보드)가 구현해야 하는 트레이트.
    /// 이 트레이트만 구현하면 어떤 게임이든 우리 검색 알고리즘을 쓸 수 있습니다.
    pub trait GameState: Clone {
        type Move: GameMove;

        fn get_legal_moves(&mut self) -> Vec<Self::Move>;
        fn make_move(&self, m: &Self::Move) -> Self;
        fn is_terminal(&self) -> bool;
        fn evaluate(&mut self) -> i32;

        /// 수 정렬을 위한 휴리스틱 점수 반환 (높을수록 먼저 탐색).
        /// killer/history 보너스는 search 모듈에서 별도 가산합니다.
        fn score_move(&self, m: &Self::Move) -> i32;

        /// 트랜스포지션 테이블용 보드 해시. 0이면 TT를 사용하지 않습니다.
        fn board_hash(&self) -> u64 { 0 }

        /// 조용하지 않은(loud) 수인지 판단: 기본 구현은 score_move > 500.
        /// 캡처·프로모션이 해당하며, quiescence search에서 확장 대상을 결정합니다.
        fn is_capture(&self, m: &Self::Move) -> bool {
            self.score_move(m) > 500
        }

        /// Delta pruning용 loud move 재료 이득 추정값 (센티폰).
        /// `i32::MAX`를 반환하면 해당 수는 항상 탐색됩니다(delta pruning 비활성).
        /// 구체적 게임 구현체에서 캡처·프로모션의 예상 재료 이득을 반환하도록 오버라이드하세요.
        fn loud_move_gain(&self, _m: &Self::Move) -> i32 {
            i32::MAX
        }

        /// SEE (Static Exchange Evaluation) 점수를 반환합니다.
        /// 양수/0: 유리하거나 동등한 교환, 음수: 손해 교환.
        /// 기본값 `i32::MAX`는 항상 탐색 (SEE 프루닝 비활성).
        fn static_exchange_evaluation_move(&self, _m: &Self::Move) -> i32 { i32::MAX }
    }

    // -------------------------------------------------------------------------
    // Board 평가 헬퍼 메서드
    // (engine_huristic::heuristics의 순수 함수를 조합해 보드 전체를 평가합니다)
    // -------------------------------------------------------------------------
    impl<'a, const MACHO: bool, const IMPRISONED: bool> Board<'a, MACHO, IMPRISONED, 8> {

        /// 모든 기물의 센티폰 가치 합산. 반환값: 백 절대 시점 (양수 = 백 우세).
        fn evaluate_material(&self) -> i32 {
            let mut score = 0;
            for x in 0..8u8 {
                for y in 0..8u8 {
                    if let Some(piece) = self.piece_on(&(x, y)) {
                        let is_white = self.color_on(&(x, y)) == Some(Color::White);
                        let value = heuristics::get_piece_value(piece);
                        let pst = heuristics::pst_bonus(piece, is_white, x, y);
                        if is_white {
                            score += value + pst;
                        } else {
                            score -= value + pst;
                        }
                    }
                }
            }
            score
        }

        /// 폰 쉴드 보너스 + 중앙 배치 패널티. 반환값: 백 절대 시점.
        fn evaluate_king_safety(&self) -> i32 {
            let mut score = 0;
            if let Some((kx, ky)) = self.find_king(Color::White) {
                let shield = self.count_pawn_shield(kx, ky, Color::White);
                score += heuristics::pawn_shield_bonus(shield);
                score -= heuristics::king_center_penalty(kx);
            }
            if let Some((kx, ky)) = self.find_king(Color::Black) {
                let shield = self.count_pawn_shield(kx, ky, Color::Black);
                score -= heuristics::pawn_shield_bonus(shield);
                score += heuristics::king_center_penalty(kx);
            }
            score
        }

        /// 전방에 적 폰이 없는 폰에 랭크 기반 보너스. 반환값: 백 절대 시점.
        fn evaluate_passed_pawns(&self) -> i32 {
            let mut score = 0;
            for x in 0..8u8 {
                for y in 0..8u8 {
                    if self.piece_on(&(x, y)) != Some("pawn") { continue; }
                    let Some(color) = self.color_on(&(x, y)) else { continue; };
                    if self.is_passed_pawn(x, y, color) {
                        // 홈 랭크 기준 전진 수: 백 홈=y6, 흑 홈=y1
                        let ranks_advanced = if color == Color::White {
                            6u8.saturating_sub(y)
                        } else {
                            y.saturating_sub(1)
                        };
                        let bonus = heuristics::passed_pawn_rank_bonus(ranks_advanced);
                        if color == Color::White { score += bonus; } else { score -= bonus; }
                    }
                }
            }
            score
        }

        /// d4/e4/d5/e5 점령 보너스. 반환값: 백 절대 시점.
        fn evaluate_center_control(&self) -> i32 {
            let mut score = 0;
            for &x in &[3u8, 4u8] {
                for &y in &[3u8, 4u8] {
                    if let Some(color) = self.color_on(&(x, y)) {
                        let bonus = heuristics::center_control_bonus(x, y);
                        if color == Color::White { score += bonus; } else { score -= bonus; }
                    }
                }
            }
            score
        }

        /// 위 6개 함수를 합산한 보드 점수 (백 절대 시점).
        fn evaluate_board(&mut self) -> i32 {
            //1 부분이 각 휴리스틱의 가중치
            let material    = 1 * self.evaluate_material();
            let king_safety = 1 * self.evaluate_king_safety();
            let passed      = 1 * self.evaluate_passed_pawns();
            let center      = 1 * self.evaluate_center_control();
            material + king_safety + passed + center
        }

        // --- 유틸리티 헬퍼 ------------------------------------------------------

        fn find_king(&self, color: Color) -> Option<(u8, u8)> {
            for x in 0..8u8 {
                for y in 0..8u8 {
                    if self.piece_on(&(x, y)) == Some("king")
                        && self.color_on(&(x, y)) == Some(color)
                    {
                        return Some((x, y));
                    }
                }
            }
            None
        }

        /// 킹 앞 1~2랭크, 좌우 1파일 내의 아군 폰 수.
        fn count_pawn_shield(&self, kx: u8, ky: u8, color: Color) -> u8 {
            let mut count = 0u8;
            // 백: y 감소 방향(전진), 흑: y 증가 방향(전진)
            let dy: i8 = if color == Color::White { -1 } else { 1 };
            for dx in -1i8..=1 {
                for dist in 1i8..=2 {
                    let nx = kx as i8 + dx;
                    let ny = ky as i8 + dy * dist;
                    if nx < 0 || nx > 7 || ny < 0 || ny > 7 { continue; }
                    if self.piece_on(&(nx as u8, ny as u8)) == Some("pawn")
                        && self.color_on(&(nx as u8, ny as u8)) == Some(color)
                    {
                        count += 1;
                    }
                }
            }
            count
        }

        /// 전방 파일(px-1..=px+1)에 적 폰이 없으면 패스트 폰으로 판정.
        fn is_passed_pawn(&self, px: u8, py: u8, color: Color) -> bool {
            let enemy = color.invert();
            let (y_start, y_end): (u8, u8) = match color {
                Color::White => (0, py.saturating_sub(1)),
                Color::Black => (py + 1, 7),
            };
            // y_start > y_end 이면 범위가 비어 루프를 돌지 않습니다(u8 안전).
            if y_start > y_end { return true; }
            for x in px.saturating_sub(1)..=(px + 1).min(7) {
                for y in y_start..=y_end {
                    if self.piece_on(&(x, y)) == Some("pawn")
                        && self.color_on(&(x, y)) == Some(enemy)
                    {
                        return false;
                    }
                }
            }
            true
        }

        /// `sq`를 공격하는 `color` 진영 기물의 (가치, 위치) 목록을 반환합니다.
        /// X-ray(간접 공격선)는 고려하지 않는 간이 구현입니다.
        fn get_attackers_of(&self, sq: (u8, u8), color: Color) -> Vec<(i32, (u8, u8))> {
            let mut result = Vec::new();
            let (sx, sy) = (sq.0 as i8, sq.1 as i8);

            // 폰: 백은 아래(sy+1)에서, 흑은 위(sy-1)에서 대각 공격
            let pawn_dy: i8 = if color == Color::White { 1 } else { -1 };
            for &dx in &[-1i8, 1i8] {
                let (px, py) = (sx + dx, sy + pawn_dy);
                if px >= 0 && px < 8 && py >= 0 && py < 8 {
                    let pos = (px as u8, py as u8);
                    if self.piece_on(&pos) == Some("pawn") && self.color_on(&pos) == Some(color) {
                        result.push((100, pos));
                    }
                }
            }

            // 나이트
            for &(dx, dy) in &[(-2i8,-1i8),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)] {
                let (px, py) = (sx + dx, sy + dy);
                if px >= 0 && px < 8 && py >= 0 && py < 8 {
                    let pos = (px as u8, py as u8);
                    if self.piece_on(&pos) == Some("knight") && self.color_on(&pos) == Some(color) {
                        result.push((320, pos));
                    }
                }
            }

            // 비숍·퀀 대각선
            for &(dx, dy) in &[(-1i8,-1i8),(-1,1),(1,-1),(1,1)] {
                let (mut px, mut py) = (sx + dx, sy + dy);
                while px >= 0 && px < 8 && py >= 0 && py < 8 {
                    let pos = (px as u8, py as u8);
                    if let Some(p) = self.piece_on(&pos) {
                        if self.color_on(&pos) == Some(color) {
                            match p {
                                "bishop" => result.push((330, pos)),
                                "queen"  => result.push((900, pos)),
                                _        => {}
                            }
                        }
                        break;
                    }
                    px += dx; py += dy;
                }
            }

            // 룩·퀀 직선
            for &(dx, dy) in &[(-1i8,0i8),(1,0),(0,-1),(0,1)] {
                let (mut px, mut py) = (sx + dx, sy + dy);
                while px >= 0 && px < 8 && py >= 0 && py < 8 {
                    let pos = (px as u8, py as u8);
                    if let Some(p) = self.piece_on(&pos) {
                        if self.color_on(&pos) == Some(color) {
                            match p {
                                "rook"  => result.push((500, pos)),
                                "queen" => result.push((900, pos)),
                                _       => {}
                            }
                        }
                        break;
                    }
                    px += dx; py += dy;
                }
            }

            // 킹
            for dx in -1i8..=1 {
                for dy in -1i8..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let (px, py) = (sx + dx, sy + dy);
                    if px >= 0 && px < 8 && py >= 0 && py < 8 {
                        let pos = (px as u8, py as u8);
                        if self.piece_on(&pos) == Some("king") && self.color_on(&pos) == Some(color) {
                            result.push((20_000, pos));
                        }
                    }
                }
            }

            result
        }

        /// Static Exchange Evaluation: `from` → `to` 캡처 교환 시퀀스의 재료 손익을 계산합니다.
        /// 양수 = 유리, 0 = 동등, 음수 = 불리. X-ray 없는 간이 구현.
        fn static_exchange_evaluation(&self, from: (u8, u8), to: (u8, u8)) -> i32 {
            let Some(our_color) = self.color_on(&from) else { return 0; };
            let opp_color = our_color.invert();

            let captured_val = match self.piece_on(&to) {
                Some(p) => heuristics::get_piece_value(p),
                None    => return 0,
            };
            let attacker_val = heuristics::get_piece_value(
                self.piece_on(&from).unwrap_or("pawn")
            );

            // 우리 측 후속 공격자 (from 제외, 오름차순)
            let mut our_atts: Vec<i32> = self.get_attackers_of(to, our_color)
                .into_iter()
                .filter(|&(_, pos)| pos != from)
                .map(|(v, _)| v)
                .collect();
            // 상대 측 공격자 (오름차순)
            let mut opp_atts: Vec<i32> = self.get_attackers_of(to, opp_color)
                .into_iter()
                .map(|(v, _)| v)
                .collect();
            our_atts.sort_unstable();
            opp_atts.sort_unstable();

            // gain[i]: i번째 교환에서 해당 플레이어가 획득하는 기물 가치
            let mut gain: Vec<i32> = Vec::with_capacity(16);
            gain.push(captured_val);

            let mut cur_val = attacker_val;
            let mut our_i = 0usize;
            let mut opp_i = 0usize;
            let mut opp_turn = true;

            loop {
                if opp_turn {
                    if opp_i >= opp_atts.len() {
                        break; 
                    }
                    gain.push(cur_val);
                    cur_val = opp_atts[opp_i]; 
                    opp_i += 1;
                } else {
                    if our_i >= our_atts.len() { 
                        break; 
                    }
                    gain.push(cur_val);
                    cur_val = our_atts[our_i]; 
                    our_i += 1;
                }
                opp_turn = !opp_turn;
            }

            // 역방향 전파: 각 플레이어는 손해면 교환 거부 가능
            // result = gain[0] - max(0, gain[1] - max(0, gain[2] - ...))
            let mut running = 0i32;
            for &g in gain.iter().skip(1).rev() {
                running = (g - running).max(0);
            }
            gain[0] - running
        }
    }

    // --- 표준 체스를 위한 GameState 구현 -------------------------------------
    impl<'a, const MACHO: bool, const IMPRISONED: bool> GameState for Board<'a, MACHO, IMPRISONED, 8> {
        type Move = ChessMove<'a>;

        fn get_legal_moves(&mut self) -> Vec<Self::Move> {
            MoveGen::new_legal(self)
        }

        fn make_move(&self, m: &Self::Move) -> Self {
            self.make_move_new(&m)
        }

        fn is_terminal(&self) -> bool {
            self.status() != BoardStatus::Ongoing
        }

        fn evaluate(&mut self) -> i32 {
            // 1. 게임 종료 상태 확인
            if self.is_terminal() {
                return match self.status() {
                    BoardStatus::Checkmate => -1_000_000,
                    BoardStatus::Stalemate => 0,
                    _ => 0,
                };
            }

            // 2. 보드 전체 평가 (백 절대 시점)
            let score = self.evaluate_board();

            // 3. 현재 플레이어 시점으로 변환
            if self.side_to_move() == Color::White { score } else { -score }
        }

        fn score_move(&self, m: &Self::Move) -> i32 {
            let mut score = 0;

            // 1. 프로모션
            if let Some(promoted_piece) = m.get_promotion() {
                score += heuristics::score_promotion(promoted_piece);
            }

            // 2. SEE 반영
            score += self.static_exchange_evaluation_move(m);

            // 3. 캡처 (MVV-LVA)
            if let Some(victim) = self.piece_on(&m.get_dest()) {
                let attacker = self.piece_on(&m.get_source()).unwrap_or("pawn");
                score += heuristics::score_capture_mvv_lva(attacker, victim);
            }

            // 4. 센터 접근 보너스
            score += heuristics::score_center_approach(m.get_source(), m.get_dest());

            // NOTE: killer/history 보너스는 search 모듈에서 KillerTable/HistoryTable로 가산.
            score
        }

        fn board_hash(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;
            let mut h = DefaultHasher::new();
            for x in 0..8u8 {
                for y in 0..8u8 {
                    self.piece_on(&(x, y)).hash(&mut h);
                    self.color_on(&(x, y)).hash(&mut h);
                }
            }
            self.side_to_move().hash(&mut h);
            h.finish()
        }

        fn is_capture(&self, m: &Self::Move) -> bool {
            self.piece_on(&m.get_dest()).is_some() || m.get_promotion().is_some()
        }

        fn loud_move_gain(&self, m: &Self::Move) -> i32 {
            let mut gain = 0i32;
            if let Some(victim) = self.piece_on(&m.get_dest()) {
                gain += heuristics::get_piece_value(victim);
            }
            if let Some(promo) = m.get_promotion() {
                gain += heuristics::get_piece_value(promo)
                    - heuristics::get_piece_value("pawn");
            }
            gain
        }

        fn static_exchange_evaluation_move(&self, m: &Self::Move) -> i32 {
            self.static_exchange_evaluation(m.get_source(), m.get_dest())
        }
    }

}

// -----------------------------------------------------------------------------
// 모듈 2: 알파-베타 검색 (네가맥스 구현)
// -----------------------------------------------------------------------------
pub mod search {
    use std::collections::HashMap;
    use rand::seq::SliceRandom;

    use super::game_logic::{GameMove, GameState};

    /// 기본 탐색 깊이. `find_best_move` 호출 시 이 값을 전달하면 됩니다.
    pub const SEARCH_DEPTH: u8 = 3;
    /// 재귀 폭발 방지용 하드 깊이 상한. (debug.html max="6", main.rs depth 검증도 이 값 기준)
    pub const HARD_DEPTH: u8 = 6;
    /// Quiescence search 최대 추가 깊이. 캡처 폭발 방지용 상한.
    pub const QUIESCENCE_DEPTH: u8 = 6;
    /// Delta pruning 안전 마진 (센티폰).
    /// stand_pat + 캡처 이득 + DELTA_MARGIN ≤ alpha 이면 해당 수를 건너뜁니다.
    const DELTA_MARGIN: i32 = 200;
    /// Aspiration Window 초기 창 크기 (센티폰).
    /// 반복 심화의 depth > 2 부터 이전 점수 ±ASPIRATION_DELTA 창으로 탐색을 시작합니다.
    const ASPIRATION_DELTA: i32 = 50;

    // -------------------------------------------------------------------------
    // 킬러 테이블: 깊이별로 베타 컷오프를 유발한 조용한 수 2개 저장.
    // -------------------------------------------------------------------------
    struct KillerTable<M: Clone + PartialEq> {
        table: Vec<[Option<M>; 2]>,
    }

    impl<M: Clone + PartialEq> KillerTable<M> {
        fn new(max_depth: usize) -> Self {
            Self { table: vec![[None, None]; max_depth + 1] }
        }

        fn store(&mut self, depth: u8, m: &M) {
            let d = depth as usize;
            if d >= self.table.len() { return; }
            // 중복 저장 방지 후 슬롯 0에 최신 킬러를 보관
            if self.table[d][0].as_ref() != Some(m) {
                self.table[d][1] = self.table[d][0].take();
                self.table[d][0] = Some(m.clone());
            }
        }

        fn get_bonus(&self, depth: u8, m: &M) -> i32 {
            let d = depth as usize;
            if d >= self.table.len() { return 0; }
            if self.table[d][0].as_ref() == Some(m)
                || self.table[d][1].as_ref() == Some(m)
            {
                9_000
            } else {
                0
            }
        }
    }

    // -------------------------------------------------------------------------
    // 히스토리 테이블: (from, to) 해시 → 누적 컷오프 점수.
    // -------------------------------------------------------------------------
    struct HistoryTable {
        scores: HashMap<u64, i32>,
    }

    impl HistoryTable {
        fn new() -> Self { Self { scores: HashMap::new() } }

        /// 깊이²를 가중치로 합산.
        fn update(&mut self, key: u64, depth: u8) {
            if key == 0 { return; }
            *self.scores.entry(key).or_insert(0) += (depth as i32) * (depth as i32);
        }

        fn get(&self, key: u64) -> i32 {
            if key == 0 { return 0; }
            *self.scores.get(&key).unwrap_or(&0)
        }
    }

    // -------------------------------------------------------------------------
    // 트랜스포지션 테이블: 보드 해시 → (깊이, 점수, 노드 타입) 캐시.
    // -------------------------------------------------------------------------
    #[derive(Clone, Copy, PartialEq)]
    enum NodeType {
        /// 알파-베타 창 내부: 정확한 점수.
        Exact,
        /// 베타 컷오프: 실제 점수는 저장값 이상 (하한).
        LowerBound,
        /// 알파 미달: 실제 점수는 저장값 이하 (상한).
        UpperBound,
    }

    struct TtEntry {
        depth: u8,
        score: i32,
        node_type: NodeType,
        best_move: u64,  // 이 노드에서 찾은 최선 수의 move_hash(), 0이면 없음
    }

    struct TranspositionTable {
        table: HashMap<u64, TtEntry>,
    }

    impl TranspositionTable {
        fn new() -> Self { Self { table: HashMap::new() } }

        /// 캐시 히트 시 바로 반환할 수 있는 점수를 돌려줌.
        fn probe(&self, key: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
            let e = self.table.get(&key)?;
            if e.depth < depth { return None; }
            match e.node_type {
                NodeType::Exact      => Some(e.score),
                NodeType::LowerBound => if e.score >= beta  { Some(e.score) } else { None },
                NodeType::UpperBound => if e.score <= alpha { Some(e.score) } else { None },
            }
        }

        /// 더 깊은 탐색 결과로 엔트리를 교체(깊이 우선 교체 전략).
        fn store(&mut self, key: u64, depth: u8, score: i32, node_type: NodeType, best_move: u64) {
            let replace = match self.table.get(&key) {
                None    => true,
                Some(e) => depth >= e.depth,
            };
            if replace {
                self.table.insert(key, TtEntry { depth, score, node_type, best_move });
            }
        }

        /// 저장된 최선 수의 move_hash()를 반환합니다. 엔트리가 없으면 0.
        /// 깊이 조건 없이 항상 반환 (수 정렬 전용).
        fn get_best_move(&self, key: u64) -> u64 {
            self.table.get(&key).map_or(0, |e| e.best_move)
        }
    }

    // -------------------------------------------------------------------------
    // 공개 API (시그니처 변경 없음)
    // -------------------------------------------------------------------------
    pub fn find_best_move<S: GameState>(state: &mut S, depth: u8, beam_width: Option<usize>) -> Result<(S::Move, i32), usize> {
        if state.is_terminal() || depth == 0 {
            return Err(260);
        }

        let mut killers = KillerTable::new(depth as usize + 2);
        let mut history = HistoryTable::new();
        let mut tt      = TranspositionTable::new();

        // killers/history/TT는 반복 전체에서 공유해 수 정렬 품질을 높입니다.
        let mut best_move: Option<S::Move> = None;
        let mut best_score = -i32::MAX;

        let mut rng = rand::rng();
        let n = state.get_legal_moves().len();

        // 반복 심화(Iterative Deepening): 깊이 1 → target_depth 순차 탐색.
        // Aspiration Window: depth > 2 부터 이전 점수 ±ASPIRATION_DELTA 창으로 탐색.
        // 창 밖(fail-low/fail-high)이면 창을 4배씩 넓혀 재탐색합니다.
        for current_depth in 1..=depth {
            let use_aspiration = current_depth > 2
                && best_score != -i32::MAX
                && best_score.abs() < 900_000;
            let mut asp_delta = ASPIRATION_DELTA;
            let mut asp_lo = if use_aspiration { best_score - asp_delta } else { -i32::MAX };
            let mut asp_hi = if use_aspiration { best_score + asp_delta } else { i32::MAX };

            'aspiration: loop {
                let mut alpha = asp_lo;
                let beta = asp_hi;
                let mut iter_best_move: Option<S::Move> = None;
                let mut iter_best_score = -i32::MAX;

                let mut moves = state.get_legal_moves();
                moves.shuffle(&mut rng);

                // 수 정렬: 1순위 이전 반복 최선 수(TT move 역할) → 2순위 캡처 → 3순위 킬러 → 4순위 히스토리
                let prev_best_hash = best_move.as_ref().map(|bm| bm.move_hash()).unwrap_or(0);
                moves.sort_by(|a, b| {
                    let sa = {
                        let h = a.move_hash();
                        if prev_best_hash != 0 && h == prev_best_hash {
                            2_000_000
                        } else if state.is_capture(a) {
                            1_000_000 + state.score_move(a)
                        } else if killers.get_bonus(current_depth, a) > 0 {
                            900_000 + history.get(h)
                        } else {
                            state.score_move(a) + history.get(h)
                        }
                    };
                    let sb = {
                        let h = b.move_hash();
                        if prev_best_hash != 0 && h == prev_best_hash {
                            2_000_000
                        } else if state.is_capture(b) {
                            1_000_000 + state.score_move(b)
                        } else if killers.get_bonus(current_depth, b) > 0 {
                            900_000 + history.get(h)
                        } else {
                            state.score_move(b) + history.get(h)
                        }
                    };
                    sb.cmp(&sa)
                });
                if current_depth > 2 {
                    if let Some(bw) = beam_width {
                        moves.truncate(bw);
                    }
                }

                for m in moves {
                    let mut new_state = state.make_move(&m);
                    let score = -negamax(&mut new_state, current_depth - 1, HARD_DEPTH, -beta, -alpha, beam_width, &mut killers, &mut history, &mut tt);

                    if score > iter_best_score {
                        iter_best_score = score;
                        iter_best_move = Some(m);
                    }
                    alpha = alpha.max(iter_best_score);
                }

                if let Some(m) = iter_best_move {
                    if iter_best_score <= asp_lo && asp_lo > -i32::MAX {
                        // Fail-low: 하한을 넓힘 (상한 유지)
                        asp_delta = asp_delta.saturating_mul(4);
                        asp_lo = if asp_delta > 1_500_000 { -i32::MAX } else { best_score - asp_delta };
                    } else if iter_best_score >= asp_hi && asp_hi < i32::MAX {
                        // Fail-high: 상한을 넓힘, 부분 결과로 최선 수 업데이트
                        best_move = Some(m);
                        best_score = iter_best_score;
                        asp_delta = asp_delta.saturating_mul(4);
                        asp_hi = if asp_delta > 1_500_000 { i32::MAX } else { best_score + asp_delta };
                    } else {
                        // 창 내 탐색 성공
                        best_move = Some(m);
                        best_score = iter_best_score;
                        break 'aspiration;
                    }
                } else {
                    break 'aspiration;
                }
            }
        }

        best_move.map(|m| (m, best_score)).ok_or(n)
    }

    fn negamax<S: GameState>(
        state: &mut S,
        depth: u8,
        hard_depth: u8,
        mut alpha: i32,
        beta: i32,
        beam_width: Option<usize>,
        killers: &mut KillerTable<S::Move>,
        history: &mut HistoryTable,
        tt: &mut TranspositionTable,
    ) -> i32 {
        if state.is_terminal() {
            return state.evaluate();
        }
        if depth == 0 || hard_depth == 0 {
            return quiescence_search(state, alpha, beta, QUIESCENCE_DEPTH);
        }

        // --- 트랜스포지션 테이블 조회 ---
        let orig_alpha = alpha;
        let tt_key = state.board_hash();
        if tt_key != 0 {
            if let Some(cached) = tt.probe(tt_key, depth, alpha, beta) {
                return cached;
            }
        }

        let damper = 1.0;
        let mut value = -i32::MAX;

        // TT 최선 수 해시 조회 (수 정렬 1순위용)
        let tt_move_hash: u64 = if tt_key != 0 { tt.get_best_move(tt_key) } else { 0 };

        // 수 정렬: 1순위 TT move → 2순위 캡처(MVV-LVA) → 3순위 킬러 → 4순위 히스토리
        let mut moves: Vec<_> = state
            .get_legal_moves()
            .into_iter()
            .map(|m| {
                let mhash = m.move_hash();
                let s = if tt_move_hash != 0 && mhash == tt_move_hash {
                    2_000_000
                } else if state.is_capture(&m) {
                    1_000_000 + state.score_move(&m)
                } else if killers.get_bonus(depth, &m) > 0 {
                    900_000 + history.get(mhash)
                } else {
                    state.score_move(&m) + history.get(mhash)
                };
                (s, m)
            })
            .collect();
        moves.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        if depth > 2 {
            if let Some(n) = beam_width {
                moves.truncate(n);
            }
        }
        let moves_len = moves.len();
        let mut best_move_hash: u64 = 0;

        for (i, (_, m)) in moves.into_iter().enumerate() {
            // LMR: 상위 3수는 depth-1(full), 이후 수는 depth-2(reduced)로 탐색
            let search_depth = if moves_len >= 4 && i >= 3 {
                depth.saturating_sub(2)
            } else {
                depth - 1
            };
            let mut new_state = state.make_move(&m);
            let score = -negamax(&mut new_state, search_depth, hard_depth - 1, -beta, -alpha, beam_width, killers, history, tt);

            if score > value {
                value = score;
                best_move_hash = m.move_hash();
            }
            alpha = alpha.max(value);

            if alpha >= beta {
                // 베타 컷오프: 킬러 저장 + 히스토리 업데이트
                killers.store(depth, &m);
                history.update(m.move_hash(), depth);
                break;
            }
        }

        let final_value = (value as f32 * damper) as i32;

        // --- 트랜스포지션 테이블 저장 (best move hash 포함) ---
        if tt_key != 0 {
            let node_type = if final_value <= orig_alpha {
                NodeType::UpperBound
            } else if final_value >= beta {
                NodeType::LowerBound
            } else {
                NodeType::Exact
            };
            tt.store(tt_key, depth, final_value, node_type, best_move_hash);
        }

        final_value
    }

    /// Quiescence Search: depth-0 노드에서 캡처·프로모션만 확장해
    /// 지평선 효과(horizon effect)를 완화합니다.
    ///
    /// - stand-pat(정적 평가)를 하한(alpha)로 사용.
    /// - 캡처·프로모션이 없거나 depth_limit에 도달하면 stand-pat 반환.
    fn quiescence_search<S: GameState>(
        state: &mut S,
        mut alpha: i32,
        beta: i32,
        depth_limit: u8,
    ) -> i32 {
        // 터미널 상태는 정적 평가로 처리
        if state.is_terminal() {
            return state.evaluate();
        }

        // Stand-pat: 현재 수를 두지 않는 정적 평가
        let stand_pat = state.evaluate();

        // Fail-hard 베타 컷오프
        if stand_pat >= beta {
            return beta;
        }

        // stand-pat를 알파 하한으로 설정
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // 깊이 한계 도달
        if depth_limit == 0 {
            return alpha;
        }

        // 캡처·프로모션만 필터링 (SEE < 0인 손해 교환 제거), MVV-LVA 정렬
        let mut loud_moves: Vec<_> = state
            .get_legal_moves()
            .into_iter()
            .filter(|m| state.is_capture(m) && state.static_exchange_evaluation_move(m) >= 0)
            .collect();

        loud_moves.sort_unstable_by(|a, b| {
            state.score_move(b).cmp(&state.score_move(a))
        });

        for m in loud_moves {
            // Delta pruning: 이 수로 얻을 수 있는 최대 재료 이득이
            // alpha 개선에 충분하지 않으면 건너뜁니다.
            let gain = state.loud_move_gain(&m);
            if gain != i32::MAX && stand_pat + gain + DELTA_MARGIN <= alpha {
                continue;
            }

            let mut new_state = state.make_move(&m);
            let score = -quiescence_search(&mut new_state, -beta, -alpha, depth_limit - 1);

            if score >= beta {
                return beta; // 베타 컷오프
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    // =========================================================================
    // 디버그 통계 수집용 구조체 및 공개 API
    // =========================================================================

    /// 검색 중 수집되는 내부 카운터.
    struct SearchStats {
        nodes: u64,
        qnodes: u64,
        tt_hits: u64,
        tt_lookups: u64,
        beta_cutoffs: u64,
        cutoff_first_move: u64,
        asp_retries: u64,
    }

    impl SearchStats {
        fn new() -> Self {
            Self { nodes: 0, qnodes: 0, tt_hits: 0, tt_lookups: 0, beta_cutoffs: 0, cutoff_first_move: 0, asp_retries: 0 }
        }
    }

    /// `find_best_move_debug` 가 반환하는 진단 정보 구조체.
    #[derive(serde::Serialize)]
    pub struct EngineDebugInfo<M: serde::Serialize> {
        /// 실제 탐색에 사용된 최종 깊이.
        pub depth: u8,
        /// 탐색 소요 시간 (밀리초).
        pub elapsed_ms: u64,
        /// 메인 탐색(negamax) 노드 수.
        pub nodes: u64,
        /// 정적 탐색(quiescence) 노드 수.
        pub qnodes: u64,
        /// 초당 탐색 노드 수 (nodes + qnodes) / elapsed_ms * 1000.
        pub nps: u64,
        /// 트랜스포지션 테이블 히트율 (0.0 ~ 1.0).
        pub tt_hit_rate: f64,
        /// 전체 노드 중 quiescence 노드 비율 (0.0 ~ 1.0).
        pub qnodes_ratio: f64,
        /// 베타 컷오프 발생 횟수.
        pub beta_cutoffs: u64,
        /// 베타 컷오프 중 첫 번째 수에서 발생한 비율 (이상적으로 1.0에 가까울수록 좋음).
        pub cutoff_first_rate: f64,
        /// Aspiration Window 창 초과로 재탐색한 횟수.
        pub asp_retries: u64,
        /// 탐색된 최선 수.
        pub best_move: Option<M>,
        /// 최선 수의 점수 (현재 플레이어 시점).
        pub score: i32,
    }

    /// 디버그용 최선 수 탐색.
    /// `find_best_move` 와 동일한 알고리즘(Iterative Deepening + TT + Killer + History + LMR + QS)이지만
    /// 각종 통계를 수집해 `EngineDebugInfo` 로 반환합니다.
    /// 기존 `find_best_move` 는 변경하지 않습니다.
    pub fn find_best_move_debug<S>(
        state: &mut S,
        depth: u8,
        beam_width: Option<usize>,
    ) -> EngineDebugInfo<S::Move>
    where
        S: GameState,
        S::Move: serde::Serialize,
    {
        let start = std::time::Instant::now();
        let mut stats = SearchStats::new();

        if state.is_terminal() || depth == 0 {
            return EngineDebugInfo {
                depth, elapsed_ms: 0, nodes: 0, qnodes: 0, nps: 0,
                tt_hit_rate: 0.0, qnodes_ratio: 0.0, beta_cutoffs: 0,
                cutoff_first_rate: 0.0, asp_retries: 0, best_move: None, score: 0,
            };
        }

        let mut killers = KillerTable::new(depth as usize + 2);
        let mut history = HistoryTable::new();
        let mut tt      = TranspositionTable::new();

        let mut best_move: Option<S::Move> = None;
        let mut best_score = -i32::MAX;

        let mut rng = rand::rng();

        for current_depth in 1..=depth {
            let use_aspiration = current_depth > 2
                && best_score != -i32::MAX
                && best_score.abs() < 900_000;
            let mut asp_delta = ASPIRATION_DELTA;
            let mut asp_lo = if use_aspiration { best_score - asp_delta } else { -i32::MAX };
            let mut asp_hi = if use_aspiration { best_score + asp_delta } else { i32::MAX };

            'aspiration: loop {
                let mut alpha = asp_lo;
                let beta = asp_hi;
                let mut iter_best_move: Option<S::Move> = None;
                let mut iter_best_score = -i32::MAX;

                let mut moves = state.get_legal_moves();
                moves.shuffle(&mut rng);

                // 수 정렬: 1순위 이전 반복 최선 수(TT move 역할) → 2순위 캡처 → 3순위 킬러 → 4순위 히스토리
                let prev_best_hash = best_move.as_ref().map(|bm| bm.move_hash()).unwrap_or(0);
                moves.sort_by(|a, b| {
                    let sa = {
                        let h = a.move_hash();
                        if prev_best_hash != 0 && h == prev_best_hash {
                            2_000_000
                        } else if state.is_capture(a) {
                            1_000_000 + state.score_move(a)
                        } else if killers.get_bonus(current_depth, a) > 0 {
                            900_000 + history.get(h)
                        } else {
                            state.score_move(a) + history.get(h)
                        }
                    };
                    let sb = {
                        let h = b.move_hash();
                        if prev_best_hash != 0 && h == prev_best_hash {
                            2_000_000
                        } else if state.is_capture(b) {
                            1_000_000 + state.score_move(b)
                        } else if killers.get_bonus(current_depth, b) > 0 {
                            900_000 + history.get(h)
                        } else {
                            state.score_move(b) + history.get(h)
                        }
                    };
                    sb.cmp(&sa)
                });
                if current_depth > 2 {
                    if let Some(bw) = beam_width {
                        moves.truncate(bw);
                    }
                }

                for m in moves {
                    let mut new_state = state.make_move(&m);
                    let score = -negamax_debug(
                        &mut new_state, current_depth - 1, HARD_DEPTH,
                        -beta, -alpha, beam_width,
                        &mut killers, &mut history, &mut tt, &mut stats,
                    );

                    if score > iter_best_score {
                        iter_best_score = score;
                        iter_best_move = Some(m);
                    }
                    alpha = alpha.max(iter_best_score);
                }

                if let Some(m) = iter_best_move {
                    if iter_best_score <= asp_lo && asp_lo > -i32::MAX {
                        // Fail-low: 하한을 넓힘 (상한 유지)
                        stats.asp_retries += 1;
                        asp_delta = asp_delta.saturating_mul(4);
                        asp_lo = if asp_delta > 1_500_000 { -i32::MAX } else { best_score - asp_delta };
                    } else if iter_best_score >= asp_hi && asp_hi < i32::MAX {
                        // Fail-high: 상한을 넓힘, 부분 결과로 최선 수 업데이트
                        stats.asp_retries += 1;
                        best_move = Some(m);
                        best_score = iter_best_score;
                        asp_delta = asp_delta.saturating_mul(4);
                        asp_hi = if asp_delta > 1_500_000 { i32::MAX } else { best_score + asp_delta };
                    } else {
                        best_move = Some(m);
                        best_score = iter_best_score;
                        break 'aspiration;
                    }
                } else {
                    break 'aspiration;
                }
            }
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        let total_nodes = stats.nodes + stats.qnodes;
        let nps = if elapsed_ms > 0 { total_nodes * 1000 / elapsed_ms } else { total_nodes };

        EngineDebugInfo {
            depth,
            elapsed_ms,
            nodes: stats.nodes,
            qnodes: stats.qnodes,
            nps,
            tt_hit_rate:       if stats.tt_lookups   > 0 { stats.tt_hits         as f64 / stats.tt_lookups   as f64 } else { 0.0 },
            qnodes_ratio:      if total_nodes         > 0 { stats.qnodes          as f64 / total_nodes        as f64 } else { 0.0 },
            beta_cutoffs:      stats.beta_cutoffs,
            cutoff_first_rate: if stats.beta_cutoffs  > 0 { stats.cutoff_first_move as f64 / stats.beta_cutoffs as f64 } else { 0.0 },
            asp_retries:       stats.asp_retries,
            best_move,
            score: best_score,
        }
    }

    /// 통계 추적 버전 negamax (find_best_move_debug 전용).
    fn negamax_debug<S: GameState>(
        state: &mut S,
        depth: u8,
        hard_depth: u8,
        mut alpha: i32,
        beta: i32,
        beam_width: Option<usize>,
        killers: &mut KillerTable<S::Move>,
        history: &mut HistoryTable,
        tt: &mut TranspositionTable,
        stats: &mut SearchStats,
    ) -> i32 {
        stats.nodes += 1;

        if state.is_terminal() {
            return state.evaluate();
        }
        if depth == 0 || hard_depth == 0 {
            return quiescence_search_debug(state, alpha, beta, QUIESCENCE_DEPTH, stats);
        }

        let orig_alpha = alpha;
        let tt_key = state.board_hash();
        if tt_key != 0 {
            stats.tt_lookups += 1;
            if let Some(cached) = tt.probe(tt_key, depth, alpha, beta) {
                stats.tt_hits += 1;
                return cached;
            }
        }

        let damper = 1.0;
        let mut value = -i32::MAX;

        // TT 최선 수 해시 조회 (수 정렬 1순위용)
        let tt_move_hash: u64 = if tt_key != 0 { tt.get_best_move(tt_key) } else { 0 };

        // 수 정렬: 1순위 TT move → 2순위 캡처(MVV-LVA) → 3순위 킬러 → 4순위 히스토리
        let mut moves: Vec<_> = state
            .get_legal_moves()
            .into_iter()
            .map(|m| {
                let mhash = m.move_hash();
                let s = if tt_move_hash != 0 && mhash == tt_move_hash {
                    2_000_000
                } else if state.is_capture(&m) {
                    1_000_000 + state.score_move(&m)
                } else if killers.get_bonus(depth, &m) > 0 {
                    900_000 + history.get(mhash)
                } else {
                    state.score_move(&m) + history.get(mhash)
                };
                (s, m)
            })
            .collect();
        moves.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        if depth > 2 {
            if let Some(n) = beam_width {
                moves.truncate(n);
            }
        }
        let moves_len = moves.len();
        let mut best_move_hash: u64 = 0;

        for (i, (_, m)) in moves.into_iter().enumerate() {
            let search_depth = if moves_len >= 4 && i >= 3 {
                depth.saturating_sub(2)
            } else {
                depth - 1
            };
            let mut new_state = state.make_move(&m);
            let score = -negamax_debug(&mut new_state, search_depth, hard_depth - 1, -beta, -alpha, beam_width, killers, history, tt, stats);

            if score > value {
                value = score;
                best_move_hash = m.move_hash();
            }
            alpha = alpha.max(value);

            if alpha >= beta {
                stats.beta_cutoffs += 1;
                if i == 0 { stats.cutoff_first_move += 1; }
                killers.store(depth, &m);
                history.update(m.move_hash(), depth);
                break;
            }
        }

        let final_value = (value as f32 * damper) as i32;

        if tt_key != 0 {
            let node_type = if final_value <= orig_alpha {
                NodeType::UpperBound
            } else if final_value >= beta {
                NodeType::LowerBound
            } else {
                NodeType::Exact
            };
            tt.store(tt_key, depth, final_value, node_type, best_move_hash);
        }

        final_value
    }

    /// 통계 추적 버전 quiescence search (find_best_move_debug 전용).
    fn quiescence_search_debug<S: GameState>(
        state: &mut S,
        mut alpha: i32,
        beta: i32,
        depth_limit: u8,
        stats: &mut SearchStats,
    ) -> i32 {
        stats.qnodes += 1;

        if state.is_terminal() {
            return state.evaluate();
        }

        let stand_pat = state.evaluate();
        if stand_pat >= beta { return beta; }
        if stand_pat > alpha { alpha = stand_pat; }
        if depth_limit == 0 { return alpha; }

        let mut loud_moves: Vec<_> = state
            .get_legal_moves()
            .into_iter()
            .filter(|m| state.is_capture(m) && state.static_exchange_evaluation_move(m) >= 0)
            .collect();
        loud_moves.sort_unstable_by(|a, b| state.score_move(b).cmp(&state.score_move(a)));

        for m in loud_moves {
            // Delta pruning
            let gain = state.loud_move_gain(&m);
            if gain != i32::MAX && stand_pat + gain + DELTA_MARGIN <= alpha {
                continue;
            }

            let mut new_state = state.make_move(&m);
            let score = -quiescence_search_debug(&mut new_state, -beta, -alpha, depth_limit - 1, stats);
            if score >= beta { return beta; }
            if score > alpha { alpha = score; }
        }

        alpha
    }
}
