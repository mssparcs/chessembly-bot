use serde::Serialize;

use super::engine::game_logic::GameState;
use super::engine::search::find_best_move;

#[derive(Debug, PartialEq, Serialize)]
pub enum MoveQuality {
    Brilliant, // !! 탁월수 (기물 희생 등 얕게 보면 실수지만 깊게 보면 정수)
    Best,      // !  최선의 수
    Good,      //    좋은 수
    Inaccuracy,// ?! 부정확
    Mistake,   // ?  실수
    Blunder,   // ?? 블런더
}

/// 플레이어가 둔 수를 분석하여 등급을 판정합니다.
pub fn analyze_move<S: GameState>(
    state: &mut S,
    played_move: &S::Move,
    depth: u8,
) -> (MoveQuality, i32, i32) {
    
    // 1. [깊은 탐색] 현재 국면에서 진짜 최선의 점수 (E_best_deep)
    let (_, best_score_deep) = find_best_move(state, depth, None).unwrap_or((played_move.clone(), 0));

    // 2. 플레이어가 둔 수를 적용한 다음 상태
    let mut state_after_move = state.make_move(played_move);

    // 3. [깊은 탐색] 플레이어가 둔 수의 진짜 평가치 (E_actual_deep)
    let actual_score_deep = if state_after_move.is_terminal() {
        -state_after_move.evaluate()
    } else {
        // 다음 턴은 상대방이므로, 점수 부호를 뒤집어 줍니다 (-)
        let (_, opp_score) = find_best_move(&mut state_after_move, depth - 1, None).unwrap_or((played_move.clone(), 0));
        -opp_score 
    };

    // 4. 진짜(깊은) 점수 손실폭 계산 (Delta)
    let delta = best_score_deep - actual_score_deep;

    // 5. 1차 등급 판별 (깊은 탐색 기준)
    let mut quality = if delta >= 300 {
        MoveQuality::Blunder    // 3점 이상 손해
    } else if delta >= 100 {
        MoveQuality::Mistake    // 1점~3점 손해
    } else if delta >= 50 {
        MoveQuality::Inaccuracy // 0.5점~1점 손해
    } else if delta <= 30 {
        MoveQuality::Best       // 최선의 수 (손실 0.3점 이하)
    } else {
        MoveQuality::Good       // 그 외 무난한 수
    };

    // 6. [탁월수 판별 로직] - "이 수가 최선의 수일 때만 검사"
    // 충분한 탐색 깊이(최소 4 이상)가 보장되어야 탁월수 판별이 의미가 있습니다.
    if quality == MoveQuality::Best && depth >= 4 {
        // 얕은 깊이 (예: 전체 깊이의 절반)
        let shallow_depth = depth / 2;
        
        // 얕게 봤을 때 엔진이 생각하는 최선의 점수
        let (_, best_score_shallow) = find_best_move(state, shallow_depth, None).unwrap_or((played_move.clone(), 0));
        
        // 얕게 봤을 때 '플레이어가 둔 수'의 점수
        let actual_score_shallow = if state_after_move.is_terminal() {
            -state_after_move.evaluate()
        } else {
            let (_, opp_score) = find_best_move(&mut state_after_move, shallow_depth - 1, None).unwrap_or((played_move.clone(), 0));
            -opp_score
        };

        // 얕은 탐색 기준에서의 점수 손실폭
        let shallow_delta = best_score_shallow - actual_score_shallow;

        // 탁월수 조건:
        // 얕게 봤을 때는 기물을 잃는 블런더나 큰 실수(예: 2점(200) 이상 손해)로 보이지만,
        // 깊게 탐색한 위(4번)의 결과에서는 최선의 수(Best)인 경우.
        if shallow_delta >= 200 {
            quality = MoveQuality::Brilliant;
        }
    }

    // 결과 반환 (수 품질, 최고 기대 점수, 실제 둔 수의 기대 점수)
    (quality, best_score_deep, actual_score_deep)
}
