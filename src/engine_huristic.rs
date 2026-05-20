// -----------------------------------------------------------------------------
// 휴리스틱 모듈: evaluate 및 score_move에 사용되는 순수 휴리스틱 함수들
// -----------------------------------------------------------------------------
pub mod heuristics {

    /// 기물의 점수 가치를 반환합니다.
    pub fn get_piece_value(piece: &str) -> i32 {
        match piece {
            "pawn"   => 1,
            "knight" => 3,
            "bishop" => 3,
            "rook"   => 5,
            "queen"  => 9,
            "king"   => 10000,
            _        => 8,
        }
    }

    /// MVV-LVA (Most Valuable Victim, Least Valuable Attacker) 캡처 점수.
    /// 예: 폰으로 퀸 잡기: (9 * 50) - (1 * 5) = 445
    /// 예: 퀸으로 폰 잡기: (1 * 50) - (9 * 5) = 5
    pub fn score_capture(attacker: &str, victim: &str) -> i32 {
        get_piece_value(victim) * 50 - get_piece_value(attacker) * 5
    }

    /// 프로모션 점수.
    pub fn score_promotion(promoted_piece: &str) -> i32 {
        50 + 5 * get_piece_value(promoted_piece)
    }

    /// 센터 근접 점수: 이동 전후 중앙 거리 차이를 기물 가치로 나눈 값.
    /// 중앙(3.5, 3.5)까지의 맨해튼 거리 근사: |2x - 7| + |2y - 7|
    /// 인수: 출발 칸, 도착 칸, 이동하는 기물 이름
    pub fn score_center_approach(src: (u8, u8), dst: (u8, u8), piece: &str) -> i32 {
        fn center_dist(pos: (u8, u8)) -> i32 {
            (pos.0 as i32 * 2 - 7).abs() + (pos.1 as i32 * 2 - 7).abs()
        }
        let piece_val = get_piece_value(piece).max(1);
        (center_dist(src) - center_dist(dst)) / piece_val
    }
}
