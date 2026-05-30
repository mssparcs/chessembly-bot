use super::{
    DeltaPosition,
    WallCollision,
    JitContext,
    ChessMove,
    MoveType,
    PieceSpan,
    Position,
    Board,
    Color,
};
use crate::chessembly::ChessMoveUnit;
use std::cmp::Ordering;

fn wall_collision<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(anchor: &Position, delta: &DeltaPosition, board: &Board<MACHO, IMPRISONED, SIZE>, color: Color) -> WallCollision {
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

fn is_friendly<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(pos: Position, board: &Board<'a, MACHO, IMPRISONED, SIZE>, color: Color) -> bool {
    if let PieceSpan::Piece(p) = &board.board[pos.1 as usize][pos.0 as usize] {
        p.color == color
    } else {
        false
    }
}

fn is_enemy<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(pos: Position, board: &Board<'a, MACHO, IMPRISONED, SIZE>, color: Color) -> bool {
    if let PieceSpan::Piece(p) = &board.board[pos.1 as usize][pos.0 as usize] {
        p.color == color.invert()
    } else {
        false
    }
}

fn push_node<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx: &mut JitContext<'a, MACHO, IMPRISONED, SIZE>, move_type: MoveType, move_to: Position, take: Position) {
    let from = (ctx.start_pos_col as u8, ctx.start_pos_row as u8);
    
    let state_change = if ctx.state_change_len > 0 {
        let mut sc_vec = Vec::new();
        for i in 0..ctx.state_change_len as usize {
            let key_slice = unsafe {
                std::slice::from_raw_parts(ctx.state_change_keys[i], ctx.state_change_key_lens[i] as usize)
            };
            let key = std::str::from_utf8(key_slice).unwrap_or("");
            sc_vec.push((key, ctx.state_change_vals[i]));
        }
        Some(sc_vec)
    } else {
        None
    };

    let transition = if !ctx.transition_ptr.is_null() {
        let tr_slice = unsafe {
            std::slice::from_raw_parts(ctx.transition_ptr, ctx.transition_len as usize)
        };
        std::str::from_utf8(tr_slice).ok()
    } else {
        None
    };

    unsafe {
        (*ctx.nodes).push(ChessMove::Single(ChessMoveUnit {
            from,
            take,
            move_to,
            move_type,
            state_change,
            transition,
        }));
    }
}

pub extern "C" fn rust_helper_should_skip<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) -> bool {
    let ctx = unsafe { &*ctx_ptr };
    !ctx.last_state()
}

pub extern "C" fn jit_helper_not<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    let state = ctx.last_state();
    ctx.set_last_state(!state);
}

pub extern "C" fn jit_helper_block_open<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, close_index: u64) {
    let ctx = unsafe { &mut *ctx_ptr };
    let cur = ctx.current_position();
    
    let len = ctx.position_stack_len as usize;
    ctx.position_stack_cols[len] = cur.0 as u64;
    ctx.position_stack_rows[len] = cur.1 as u64;
    ctx.position_stack_closes[len] = close_index;
    ctx.position_stack_len += 1;

    let t_len = ctx.take_stack_len as usize;
    if t_len > 0 && ctx.take_stack_has_value[t_len - 1] == 1 {
        ctx.take_stack_cols[t_len] = ctx.take_stack_cols[t_len - 1];
        ctx.take_stack_rows[t_len] = ctx.take_stack_rows[t_len - 1];
        ctx.take_stack_has_value[t_len] = 1;
    } else {
        ctx.take_stack_has_value[t_len] = 0;
    }
    ctx.take_stack_len += 1;

    let s_len = ctx.states_len as usize;
    ctx.states[s_len] = 1; // push true
    ctx.states_len += 1;
}

pub extern "C" fn jit_helper_block_close<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    if ctx.position_stack_len > 1 {
        ctx.position_stack_len -= 1;
    }
    if ctx.take_stack_len > 1 {
        ctx.take_stack_len -= 1;
    }
    if ctx.states_len > 1 {
        ctx.states_len -= 1;
    }
}

pub extern "C" fn jit_helper_move<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;

    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }
    let target_pos = (nx as u8, ny as u8);

    if is_friendly(target_pos, board, color_on_start) {
        ctx.set_last_state(false);
    } else if is_enemy(target_pos, board, color_on_start) {
        ctx.set_last_state(false);
    } else {
        ctx.set_current_position(target_pos);
        push_node(ctx, MoveType::Move, target_pos, target_pos);
    }
}

pub extern "C" fn jit_helper_take<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;

    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }
    let target_pos = (nx as u8, ny as u8);
    
    if is_friendly(target_pos, board, color_on_start) {
        ctx.set_last_state(false);
    } else if is_enemy(target_pos, board, color_on_start) {
        ctx.set_current_position(target_pos);
        push_node(ctx, MoveType::Take, target_pos, target_pos);
        let t_idx = (ctx.take_stack_len - 1) as usize;
        ctx.take_stack_cols[t_idx] = target_pos.0 as u64;
        ctx.take_stack_rows[t_idx] = target_pos.1 as u64;
        ctx.take_stack_has_value[t_idx] = 1;
    } else {
        ctx.set_current_position(target_pos);
    }
}

pub extern "C" fn jit_helper_take_move<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;
    
    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }
    let target_pos = (nx as u8, ny as u8);
    
    if is_friendly(target_pos, board, color_on_start) {
        ctx.set_last_state(false);
    } else if is_enemy(target_pos, board, color_on_start) {
        ctx.set_current_position(target_pos);
        push_node(ctx, MoveType::TakeMove, target_pos, target_pos);
        ctx.set_last_state(false);
    } else {
        ctx.set_current_position(target_pos);
        push_node(ctx, MoveType::TakeMove, target_pos, target_pos);
    }
}

pub extern "C" fn jit_helper_jump<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let t_idx = (ctx.take_stack_len - 1) as usize;

    if ctx.take_stack_has_value[t_idx] == 1 {
        let tp = (ctx.take_stack_cols[t_idx] as u8, ctx.take_stack_rows[t_idx] as u8);
        unsafe {
            let nodes = &mut *ctx.nodes;
            if let Some(pos) = nodes.iter().position(|x| match x {
                ChessMove::Single(n) => n.move_type == MoveType::Take && n.take == tp, // ?
                ChessMove::Multiple(_) => false
            }) {
                nodes.swap_remove(pos);
            }
        }

        if dx != 0 || dy != 0 {
            let cur = ctx.current_position();
            let nx = cur.0 as i8 + dx;
            let ny = cur.1 as i8 - dy;
            if nx < 0 || nx >= SIZE as i8 || ny < 0 || ny >= SIZE as i8 {
                let target_pos = (nx as u8, ny as u8);
                let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
                let wc = wall_collision(&cur, &(dx, dy), board, color_on_start);
                if wc == WallCollision::NoCollision && board.color_on(&target_pos).is_none() {
                    ctx.set_current_position(target_pos);
                    push_node(ctx, MoveType::TakeJump, target_pos, tp);
                    return;
                }
            }
        }
    }
    ctx.set_last_state(false);
}

pub extern "C" fn jit_helper_catch<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;

    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }
    let target_pos = (nx as u8, ny as u8);

    if is_friendly(target_pos, board, color_on_start) {
        ctx.set_last_state(false);
    } else if is_enemy(target_pos, board, color_on_start) {
        push_node(ctx, MoveType::Catch, (ctx.start_pos_col as u8, ctx.start_pos_row as u8), target_pos);
    } else {
        ctx.set_last_state(false);
    }
}


pub extern "C" fn jit_helper_peek<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;
    
    if nx < 0 || nx >= SIZE as i8 || ny < 0 || ny >= SIZE as i8 {
        ctx.set_last_state(false);
        return;
    }

    let target_pos = (nx as u8, ny as u8);
    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision || board.color_on(&target_pos).is_some() {
        ctx.set_last_state(false);
    } else {
        ctx.set_current_position(target_pos);
    }
}

pub extern "C" fn jit_helper_observe<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;

    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }
    
    let target_pos = (nx as u8, ny as u8);
    if board.color_on(&target_pos).is_some() {
        ctx.set_last_state(false);
    }
}

pub extern "C" fn jit_helper_color_on_white<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;
    
    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }

    let target_pos = (nx as u8, ny as u8);
    let color_on_target = board.color_on(&target_pos);
    if let Some(color) = color_on_target {
        ctx.set_last_state(color == Color::White);
    }
    else {
        ctx.set_last_state(false);
    }
}

pub extern "C" fn jit_helper_color_on_black<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, dx: i8, dy: i8) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;
    
    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }

    let target_pos = (nx as u8, ny as u8);
    let color_on_target = board.color_on(&target_pos);
    if let Some(color) = color_on_target {
        ctx.set_last_state(color == Color::Black);
    }
    else {
        ctx.set_last_state(false);
    }
}

pub extern "C" fn jit_helper_piece<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, name_ptr: *const u8, name_len: usize) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let name = unsafe { std::str::from_utf8(std::slice::from_raw_parts(name_ptr, name_len)).unwrap_or("") };
    let start_pos = (ctx.start_pos_col as u8, ctx.start_pos_row as u8);
    ctx.set_last_state(board.piece_on(&start_pos) == Some(name));
}

pub extern "C" fn jit_helper_color_white<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let start_pos = (ctx.start_pos_col as u8, ctx.start_pos_row as u8);
    ctx.set_last_state(board.color_on(&start_pos) == Some(Color::White));
}

pub extern "C" fn jit_helper_color_black<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let start_pos = (ctx.start_pos_col as u8, ctx.start_pos_row as u8);
    ctx.set_last_state(board.color_on(&start_pos) == Some(Color::Black));
}

pub extern "C" fn jit_helper_piece_on<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>, name_ptr: *const u8, name_len: usize, packed_delta: u64) {
    let ctx = unsafe { &mut *ctx_ptr };
    let board = unsafe { &*ctx.board };
    let name = unsafe { std::str::from_utf8(std::slice::from_raw_parts(name_ptr, name_len)).unwrap_or("") };
    
    // 32비트 압축 정보로부터 dx, dy 부호 정밀 해석 복원
    let dx = (packed_delta & 0xFF) as i8;
    let dy = ((packed_delta >> 8) & 0xFF) as i8;

    let current_pos = ctx.current_position();
    let nx = current_pos.0 as i8 + dx;
    let ny = current_pos.1 as i8 - dy;

    let target_pos = (nx as u8, ny as u8);
    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision {
        ctx.set_last_state(false);
        return;
    }

    ctx.set_last_state(board.piece_on(&target_pos) == Some(name));
}

pub extern "C" fn rust_helper_do<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    let len = ctx.states_len as usize;
    ctx.states[len] = 1; // push true
    ctx.states_len += 1;
}

pub extern "C" fn rust_helper_while_check<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) -> bool {
    let ctx = unsafe { &*ctx_ptr };
    ctx.last_state()
}

pub extern "C" fn rust_helper_while_exit<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    if ctx.states_len > 1 {
        ctx.states_len -= 1;
    }
}

pub extern "C" fn rust_helper_jmp_check<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) -> bool {
    let ctx = unsafe { &*ctx_ptr };
    ctx.last_state()
}

pub extern "C" fn rust_helper_jmp_reset<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
    let ctx = unsafe { &mut *ctx_ptr };
    ctx.set_last_state(true);
}

pub extern "C" fn rust_helper_jne_check<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(ctx_ptr: *mut JitContext<'a, MACHO, IMPRISONED, SIZE>) -> bool {
    let ctx = unsafe { &*ctx_ptr };
    !ctx.last_state()
}
