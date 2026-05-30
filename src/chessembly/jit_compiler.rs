use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::ptr;

use std::time::Instant;

use crate::chessembly::ChessMoveUnit;
use crate::chessembly::behavior::BehaviorChain;

use super::{
    Color,
    // Piece,
    PieceSpan,
    MoveType,
    Position,
    DeltaPosition,
    ChessMove,
    WallCollision,
    Board,
    Behavior,
    // board::BothBoardState,
    // board::BoardStatus,
    // board::BoardState
    // BehaviorChain
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChessemblyCompiled<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

// ------------------------------------------
// 2. JIT 실행 컨텍스트 (JitContext)
// ------------------------------------------
// str 기반 뚱뚱한 포인터를 피하기 위해 u8 바이트 슬라이스 원시 주소 포인터(*const u8)로 타입을 교체했습니다.
#[repr(C)]
pub struct JitContext<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize> {
    pub board: *const Board<'a, MACHO, IMPRISONED, SIZE>,                 // offset 0
    pub nodes: *mut Vec<ChessMove<'a>>,        // offset 8
    pub start_pos_col: u64,                    // offset 16
    pub start_pos_row: u64,                    // offset 24
    
    // 스택 영역을 고정된 u64 배열 크기로 캡슐화
    pub position_stack_cols: [u64; 32],        // offset 32 (32 * 8 = 256 bytes) -> ends at 288
    pub position_stack_rows: [u64; 32],        // offset 288 (ends at 544)
    pub position_stack_closes: [u64; 32],      // offset 544 (ends at 800)
    pub position_stack_len: u64,               // offset 800
    
    pub take_stack_cols: [u64; 32],            // offset 808 (ends at 1064)
    pub take_stack_rows: [u64; 32],            // offset 1064 (ends at 1320)
    pub take_stack_has_value: [u64; 32],       // offset 1320 (ends at 1576)
    pub take_stack_len: u64,                   // offset 1576
    
    pub states: [u64; 32],                     // offset 1584 (ends at 1840)
    pub states_len: u64,                       // offset 1840
    
    pub transition_ptr: *const u8,             // offset 1848 (Thin pointer로 변경하여 컴파일 에러 예방)
    pub transition_len: u64,                   // offset 1856
    
    pub state_change_keys: [*const u8; 32],    // offset 1864 (ends at 2120)
    pub state_change_key_lens: [u64; 32],      // offset 2120 (ends at 2376)
    pub state_change_vals: [u8; 32],           // offset 2376 (ends at 2408)
    pub state_change_len: u64,                 // offset 2408
}

impl<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize> JitContext<'a, MACHO, IMPRISONED, SIZE> {
    pub fn new(board: &Board<'a, MACHO, IMPRISONED, SIZE>, start_pos: Position, nodes: &mut Vec<ChessMove<'a>>) -> Self {
        let mut pos_cols = [0; 32];
        let mut pos_rows = [0; 32];
        pos_cols[0] = start_pos.0 as u64;
        pos_rows[0] = start_pos.1 as u64;

        let mut states = [0; 32];
        states[0] = 1; // states 초기값 true 로딩

        Self {
            board,
            nodes,
            start_pos_col: start_pos.0 as u64,
            start_pos_row: start_pos.1 as u64,
            position_stack_cols: pos_cols,
            position_stack_rows: pos_rows,
            position_stack_closes: [0; 32],
            position_stack_len: 1,
            take_stack_cols: [0; 32],
            take_stack_rows: [0; 32],
            take_stack_has_value: [0; 32],
            take_stack_len: 1,
            states,
            states_len: 1,
            transition_ptr: ptr::null(),
            transition_len: 0,
            state_change_keys: [ptr::null(); 32],
            state_change_key_lens: [0; 32],
            state_change_vals: [0; 32],
            state_change_len: 0,
        }
    }

    fn current_position(&self) -> Position {
        let idx = (self.position_stack_len - 1) as usize;
        (self.position_stack_cols[idx] as u8, self.position_stack_rows[idx] as u8)
    }

    fn set_current_position(&mut self, pos: Position) {
        let idx = (self.position_stack_len - 1) as usize;
        self.position_stack_cols[idx] = pos.0 as u64;
        self.position_stack_rows[idx] = pos.1 as u64;
    }

    fn last_state(&self) -> bool {
        if self.states_len > 0 {
            self.states[(self.states_len - 1) as usize] == 1
        } else {
            true
        }
    }

    fn set_last_state(&mut self, val: bool) {
        if self.states_len > 0 {
            self.states[(self.states_len - 1) as usize] = if val { 1 } else { 0 };
        }
    }
}

// ------------------------------------------
// 3. JIT 전용 고속 런타임 헬퍼 (C-ABI)
// ------------------------------------------

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

    if nx < 0 || nx >= SIZE as i8 || ny < 0 || ny >= SIZE as i8 {
        ctx.set_last_state(false);
        return;
    }
    let target_pos = (nx as u8, ny as u8);
    let color_on_start = board.color_on(&(ctx.start_pos_col as u8, ctx.start_pos_row as u8)).unwrap();
    let wc = wall_collision(&current_pos, &(dx, dy), board, color_on_start);
    if wc != WallCollision::NoCollision || board.color_on(&target_pos).is_some() {
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

// ------------------------------------------
// 4. JIT 컴파일러 구현체 (ChessemblyJitCompiler)
// ------------------------------------------
struct LabelPatch {
    source_inst_offset: usize, // 32비트 오프셋이 기록될 기계어 바이트 위치
    target_label_id: u32,
}

pub struct ChessemblyJitCompiler {
    code: Vec<u8>,
    label_offsets: HashMap<u32, usize>,
    label_patches: Vec<LabelPatch>,
}

impl ChessemblyJitCompiler {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            label_offsets: HashMap::new(),
            label_patches: Vec::new(),
        }
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    /// 인자 전달이 없는 헬퍼 함수 호출부 기계어 생성
    fn emit_call_native(&mut self, func_ptr: usize) {
        #[cfg(target_os = "windows")]
        {
            self.emit(&[0x48, 0x89, 0xd9]);          // mov rcx, rbx (첫 번째 인자에 ctx 복사)
            self.emit(&[0x48, 0x83, 0xec, 0x28]);    // sub rsp, 40 (shadow space + alignment)
        }
        #[cfg(unix)]
        {
            self.emit(&[0x48, 0x89, 0xdf]);          // mov rdi, rbx (첫 번째 인자에 ctx 복사)
            self.emit(&[0x48, 0x83, 0xec, 0x08]);    // sub rsp, 8 (align 16-bytes)
        }

        self.emit(&[0x48, 0xb8]);                    // mov rax, func_ptr
        self.emit(&(func_ptr as u64).to_le_bytes());
        self.emit(&[0xff, 0xd0]);                    // call rax

        #[cfg(target_os = "windows")]
        self.emit(&[0x48, 0x83, 0xc4, 0x28]);        // add rsp, 40
        #[cfg(unix)]
        self.emit(&[0x48, 0x83, 0xc4, 0x08]);        // add rsp, 8
    }

    /// (dx, dy) 델타 인자가 결합된 기물 연산 헬퍼 호출부 기계어 생성
    fn emit_call_3_args(&mut self, func_ptr: usize, dx: i8, dy: i8) {
        #[cfg(target_os = "windows")]
        {
            self.emit(&[0x48, 0x89, 0xd9]);          // mov rcx, rbx (ctx)
            self.emit(&[0x48, 0xc7, 0xc2]);          // mov rdx, dx (C-ABI에 맞춰 sign-extend 적용)
            self.emit(&(dx as i32).to_le_bytes());
            self.emit(&[0x49, 0xc7, 0xc0]);          // mov r8, dy
            self.emit(&(dy as i32).to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x28]);    // sub rsp, 40
        }
        #[cfg(unix)]
        {
            self.emit(&[0x48, 0x89, 0xdf]);          // mov rdi, rbx
            self.emit(&[0x48, 0xc7, 0xc6]);          // mov rsi, dx
            self.emit(&(dx as i64).to_le_bytes());
            self.emit(&[0x48, 0xc7, 0xc2]);          // mov rdx, dy
            self.emit(&(dy as i64).to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x08]);    // sub rsp, 8
        }

        self.emit(&[0x48, 0xb8]);                    // mov rax, func_ptr
        self.emit(&(func_ptr as u64).to_le_bytes());
        self.emit(&[0xff, 0xd0]);                    // call rax

        #[cfg(target_os = "windows")]
        self.emit(&[0x48, 0x83, 0xc4, 0x28]);
        #[cfg(unix)]
        self.emit(&[0x48, 0x83, 0xc4, 0x08]);
    }

    /// BlockOpen 전용 제어 호출부 기계어 생성
    fn emit_call_block_open(&mut self, func_ptr: usize, close_index: u64) {
        #[cfg(target_os = "windows")]
        {
            self.emit(&[0x48, 0x89, 0xd9]);          // mov rcx, rbx
            self.emit(&[0x48, 0xba]);                // mov rdx, close_index
            self.emit(&close_index.to_le_bytes());
            self.emit(&[0x48, 0x83, 0xec, 0x28]);    // sub rsp, 40
        }
        #[cfg(unix)]
        {
            self.emit(&[0x48, 0x89, 0xdf]);          // mov rdi, rbx
            self.emit(&[0x48, 0xbe]);                // mov rsi, close_index
            self.emit(&close_index.to_le_bytes());
            self.emit(&[0x48, 0x83, 0xec, 0x08]);    // sub rsp, 8
        }

        self.emit(&[0x48, 0xb8]);
        self.emit(&(func_ptr as u64).to_le_bytes());
        self.emit(&[0xff, 0xd0]);

        #[cfg(target_os = "windows")]
        self.emit(&[0x48, 0x83, 0xc4, 0x28]);
        #[cfg(unix)]
        self.emit(&[0x48, 0x83, 0xc4, 0x08]);
    }

    /// 기물 타입 문자열을 인자로 갖는 호출부 기계어 생성
    fn emit_call_string_helper<'a>(&mut self, func_ptr: usize, name: &'a str) {
        let name_ptr = name.as_ptr() as u64;
        let name_len = name.len() as u64;

        #[cfg(target_os = "windows")]
        {
            self.emit(&[0x48, 0x89, 0xd9]);          // mov rcx, rbx
            self.emit(&[0x48, 0xba]);                // mov rdx, name_ptr
            self.emit(&name_ptr.to_le_bytes());
            self.emit(&[0x49, 0xc7, 0xc0]);          // mov r8, name_len
            self.emit(&name_len.to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x28]);    // sub rsp, 40
        }
        #[cfg(unix)]
        {
            self.emit(&[0x48, 0x89, 0xdf]);          // mov rdi, rbx
            self.emit(&[0x48, 0xc7, 0xc6]);          // mov rsi, name_ptr
            self.emit(&name_ptr.to_le_bytes());
            self.emit(&[0x48, 0xc7, 0xc2]);          // mov rdx, name_len
            self.emit(&name_len.to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x08]);    // sub rsp, 8
        }

        self.emit(&[0x48, 0xb8]);
        self.emit(&(func_ptr as u64).to_le_bytes());
        self.emit(&[0xff, 0xd0]);

        #[cfg(target_os = "windows")]
        self.emit(&[0x48, 0x83, 0xc4, 0x28]);
        #[cfg(unix)]
        self.emit(&[0x48, 0x83, 0xc4, 0x08]);
    }

    pub fn compile<const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&mut self, chain: &BehaviorChain) -> CompiledChain {
        self.code.clear();
        self.label_offsets.clear();
        self.label_patches.clear();

        // 1. 블록 구조 정밀 사전 파싱 (실시간 states_mask 검사 후 건너뛸 targets 지정)
        let mut block_close_targets = vec![None; chain.len()];
        let mut active_blocks = Vec::new();
        for i in 0..chain.len() {
            match &chain[i] {
                Behavior::BlockOpen => {
                    active_blocks.push(i);
                }
                Behavior::BlockClose => {
                    if let Some(start) = active_blocks.pop() {
                        for j in (start + 1)..i {
                            if block_close_targets[j].is_none() {
                                block_close_targets[j] = Some(i);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // 컴파일이 완료된 JIT 전체 함수 탈출을 위한 레이블 식별자
        let epilogue_label_id = 999999u32;

        // [프롤로그]
        self.emit(&[0x53]);             // push rbx
        self.emit(&[0x41, 0x54]);       // push r12

        #[cfg(target_os = "windows")]
        self.emit(&[0x48, 0x89, 0xcb]); // mov rbx, rcx
        #[cfg(unix)]
        self.emit(&[0x48, 0x89, 0xfb]); // mov rbx, rdi

        for i in 0..chain.len() {
            let inst = &chain[i];
            
            // 모든 연산의 분기를 제어하는 Native Target 오프셋 매핑
            self.label_offsets.insert(i as u32, self.code.len());

            // 제어 분기 오프코드가 아닌 경우, states_mask 판정 후 false 상태면 block의 끝 또는 함수 끝으로 즉시 분기합니다.
            let is_control_expr = matches!(inst, 
                Behavior::While
                | Behavior::Jmp(_)
                | Behavior::Jne(_)
                | Behavior::Label(_)
                | Behavior::Not
                | Behavior::BlockClose
            );

            if !is_control_expr {
                // Call rust_helper_should_skip
                self.emit_call_native(rust_helper_should_skip::<MACHO, IMPRISONED, SIZE> as usize);
                self.emit(&[0x84, 0xc0]); // test al, al
                
                let close_target_id = if let Some(close_idx) = block_close_targets[i] {
                    close_idx as u32
                } else {
                    epilogue_label_id
                };
                
                // jnz <rel32_target> (안전한 32비트 conditional jump 적용!)
                self.emit(&[0x0f, 0x85, 0x00, 0x00, 0x00, 0x00]);
                let patch_offset = self.code.len() - 4;
                self.label_patches.push(LabelPatch {
                    source_inst_offset: patch_offset,
                    target_label_id: close_target_id,
                });
            }

            // 개별 Opcode 컴파일 기계어 빌딩
            match inst {
                Behavior::Move((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_move::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::Take((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_take::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::TakeMove((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_take_move::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::Jump((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_jump::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::Catch((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_catch::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::Peek((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_peek::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::Observe((dx, dy)) => {
                    self.emit_call_3_args(jit_helper_observe::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                }
                Behavior::Piece(name) => {
                    self.emit_call_string_helper(jit_helper_piece::<MACHO, IMPRISONED, SIZE> as usize, name);
                }
                Behavior::Not => {
                    self.emit_call_native(jit_helper_not::<MACHO, IMPRISONED, SIZE> as usize);
                }
                Behavior::BlockOpen => {
                    let close_idx = block_close_targets[i].unwrap_or(chain.len()) as u64;
                    self.emit_call_block_open(jit_helper_block_open::<MACHO, IMPRISONED, SIZE> as usize, close_idx);
                }
                Behavior::BlockClose => {
                    self.emit_call_native(jit_helper_block_close::<MACHO, IMPRISONED, SIZE> as usize);
                }
                Behavior::Do => {
                    let next_is_while = if let Some(Behavior::While) = chain.get(i + 1) { true } else { false };
                    if !next_is_while {
                        self.emit_call_native(rust_helper_do::<MACHO, IMPRISONED, SIZE> as usize);
                    }
                }
                Behavior::While => {
                    let mut do_index = None;
                    let mut ss = 0;
                    for j in (0..i).rev() {
                        if chain[j] == Behavior::While { ss += 1; }
                        else if chain[j] == Behavior::Do {
                            ss -= 1;
                            if ss == -1 {
                                do_index = Some(j);
                                break;
                            }
                        }
                    }
                    if let Some(do_idx) = do_index {
                        self.emit_call_native(rust_helper_while_check::<MACHO, IMPRISONED, SIZE> as usize);
                        self.emit(&[0x84, 0xc0]); // test al, al
                        // jnz do_idx (Do 시작점으로 점프)
                        self.emit(&[0x0f, 0x85, 0x00, 0x00, 0x00, 0x00]);
                        let patch_offset = self.code.len() - 4;
                        self.label_patches.push(LabelPatch {
                            source_inst_offset: patch_offset,
                            target_label_id: do_idx as u32,
                        });
                    }
                    self.emit_call_native(rust_helper_while_exit::<MACHO, IMPRISONED, SIZE> as usize);
                }
                Behavior::Label(label_id) => {
                    // 유니크한 레이블 네임스페이스 매핑
                    self.label_offsets.insert(*label_id as u32 + 10000, self.code.len());
                }
                Behavior::Jmp(label_id) => {
                    self.emit_call_native(rust_helper_jmp_check::<MACHO, IMPRISONED, SIZE> as usize);
                    self.emit(&[0x84, 0xc0]); // test al, al
                    // jnz target_label
                    self.emit(&[0x0f, 0x85, 0x00, 0x00, 0x00, 0x00]);
                    let patch_offset = self.code.len() - 4;
                    self.label_patches.push(LabelPatch {
                        source_inst_offset: patch_offset,
                        target_label_id: *label_id as u32 + 10000,
                    });
                    self.emit_call_native(rust_helper_jmp_reset::<MACHO, IMPRISONED, SIZE> as usize);
                }
                Behavior::Jne(label_id) => {
                    self.emit_call_native(rust_helper_jne_check::<MACHO, IMPRISONED, SIZE> as usize);
                    self.emit(&[0x84, 0xc0]); // test al, al
                    // jnz target_label
                    self.emit(&[0x0f, 0x85, 0x00, 0x00, 0x00, 0x00]);
                    let patch_offset = self.code.len() - 4;
                    self.label_patches.push(LabelPatch {
                        source_inst_offset: patch_offset,
                        target_label_id: *label_id as u32 + 10000,
                    });
                    self.emit_call_native(rust_helper_jmp_reset::<MACHO, IMPRISONED, SIZE> as usize);
                }
                Behavior::Repeat(n) => {
                    let target_idx = (i as isize - *n as isize) as u32;
                    // jmp rel32
                    self.emit(&[0xe9, 0x00, 0x00, 0x00, 0x00]);
                    let patch_offset = self.code.len() - 4;
                    self.label_patches.push(LabelPatch {
                        source_inst_offset: patch_offset,
                        target_label_id: target_idx,
                    });
                }
                _ => {}
            }
        }

        // [에필로그]
        self.label_offsets.insert(epilogue_label_id, self.code.len());
        self.emit(&[0x41, 0x5c]);       // pop r12
        self.emit(&[0x5b]);             // pop rbx
        self.emit(&[0xc3]);             // ret

        // 모든 Jne/Jmp 오프셋 최종 링킹 및 계산 적용
        self.resolve_patches();

        unsafe {
            let page_ptr = mem_utils::allocate_executable_memory(&self.code);
            CompiledChain {
                ptr: page_ptr,
                size: self.code.len(),
            }
        }
    }

    fn resolve_patches(&mut self) {
        let patches = std::mem::take(&mut self.label_patches);
        for patch in patches {
            if let Some(&target_offset) = self.label_offsets.get(&patch.target_label_id) {
                let next_inst = patch.source_inst_offset + 4;
                let rel = (target_offset as isize - next_inst as isize) as i32;
                self.code[patch.source_inst_offset..patch.source_inst_offset + 4].copy_from_slice(&rel.to_le_bytes());
            } else {
                panic!("JIT Linker Error: Label {} not found", patch.target_label_id);
            }
        }
    }
}

// ------------------------------------------
// 5. 컴파일된 기계어 실행 관리 구조체
// ------------------------------------------
pub struct CompiledChain {
    ptr: *mut c_void,
    size: usize,
}

impl CompiledChain {
    pub unsafe fn execute<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, ctx: &mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
        let func: extern "C" fn(*mut JitContext<'a, MACHO, IMPRISONED, SIZE>) = mem::transmute(self.ptr);
        func(ctx);
    }

    pub fn execute_from<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, board: &Board<'a, MACHO, IMPRISONED, SIZE>, start_pos: Position, nodes: &mut Vec<ChessMove<'a>>) {
        let mut ctx = JitContext::new(board, start_pos, nodes);

        unsafe {
            self.execute(&mut ctx);
        }
    }
}

unsafe impl Send for CompiledChain {}
unsafe impl Sync for CompiledChain {}

impl Drop for CompiledChain {
    fn drop(&mut self) {
        unsafe {
            mem_utils::free_executable_memory(self.ptr, self.size);
        }
    }
}

// ------------------------------------------
// 6. 가상 메모리 보안 정책 준수 매핑 유틸
// ------------------------------------------
mod mem_utils {
    use super::*;
    
    #[cfg(target_os = "windows")]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        // Windows API 라이브러리 연동 안전성을 위해 extern "system" 앞에 unsafe를 붙입니다.
        unsafe fn VirtualAlloc(lpAddress: *const c_void, dwSize: usize, flAllocationType: u32, flProtect: u32) -> *mut c_void;
        unsafe fn VirtualProtect(lpAddress: *const c_void, dwSize: usize, flNewProtect: u32, lpflOldProtect: *mut u32) -> i32;
        unsafe fn VirtualFree(lpAddress: *mut c_void, dwSize: usize, dwFreeType: u32) -> i32;
    }

    #[cfg(target_os = "windows")]
    pub unsafe fn allocate_executable_memory(code: &[u8]) -> *mut c_void {
        let size = code.len();
        let page = VirtualAlloc(ptr::null(), size, 0x1000 | 0x2000, 0x04); // PAGE_READWRITE
        if page.is_null() { panic!("JIT: VirtualAlloc 실패"); }
        ptr::copy_nonoverlapping(code.as_ptr(), page as *mut u8, size);
        let mut old = 0;
        VirtualProtect(page, size, 0x20, &mut old); // PAGE_EXECUTE_READ
        page
    }

    #[cfg(target_os = "windows")]
    pub unsafe fn free_executable_memory(page: *mut c_void, _size: usize) {
        VirtualFree(page, 0, 0x8000); // MEM_RELEASE
    }

    #[cfg(unix)]
    extern "C" {
        fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, offset: isize) -> *mut c_void;
        fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
        fn munmap(addr: *mut c_void, len: usize) -> i32;
    }

    #[cfg(unix)]
    pub unsafe fn allocate_executable_memory(code: &[u8]) -> *mut c_void {
        let size = code.len();
        #[cfg(target_os = "macos")]
        let map_anon = 0x1000;
        #[cfg(not(target_os = "macos"))]
        let map_anon = 0x20;
        
        let page = mmap(ptr::null_mut(), size, 0x1 | 0x2, 0x02 | map_anon, -1, 0);
        if page == !0 as *mut c_void { panic!("JIT: mmap 실패"); }
        ptr::copy_nonoverlapping(code.as_ptr(), page as *mut u8, size);
        mprotect(page, size, 0x1 | 0x4); // PROT_READ | PROT_EXEC
        page
    }

    #[cfg(unix)]
    pub unsafe fn free_executable_memory(page: *mut c_void, size: usize) {
        munmap(page, size);
    }
}

pub fn test(script_str: &str, board_str: &str) {
    println!("==============================================");
    println!("JIT 컴파일러 구동 및 검증");
    println!("==============================================");

    let script_str_compiled = crate::chessembly::ChessemblyCompiled::from_script(script_str).unwrap();
    let chains = &script_str_compiled.chains;

    const TEST_BOARD_SIZE: usize = 8;

    let start = Instant::now();
    let mut compiled_chains = Vec::new();

    for chain in chains {
        let mut compiler = ChessemblyJitCompiler::new();
        let compiled = compiler.compile::<false, false, TEST_BOARD_SIZE>(chain);
        compiled_chains.push(compiled);
    }
    let duration = start.elapsed();

    println!("컴파일 바이트 크기: {} bytes", compiled_chains.iter().map(|x| x.size).fold(0, |a, b| a + b));
    println!("컴파일 소요 시간: {:?}", duration);

    let script: crate::chessembly::ChessemblyCompiled = crate::chessembly::ChessemblyCompiled::new();
    let mut board: Board<false, false, TEST_BOARD_SIZE> = Board::<false, false, 8>::from_str(board_str, &script);
    let mut nodes = Vec::new();
    // let mut ctx = JitContext::new(&board, start_pos, &mut nodes);
    let start2 = Instant::now();
    
    for y in 0..TEST_BOARD_SIZE as u8 {
        for x in 0..TEST_BOARD_SIZE as u8 {
            for compiled in &compiled_chains {
                if let Some(_) = board.color_on(&(x as u8, y as u8)) {
                    compiled.execute_from(&board, (x, y), &mut nodes);
                }
            }
        }
    }

    let duration2 = start2.elapsed();

    println!("----------------------------------------------");
    println!("🚀 JIT 고속 실행 완료 결과");
    println!("----------------------------------------------");
    println!("생성된 체스 기물 이동 경로 개수: {} 개", nodes.len());

    let start3 = Instant::now();
    let mut nodes2 = Vec::new();
    for y in 0..TEST_BOARD_SIZE as u8 {
        for x in 0..TEST_BOARD_SIZE as u8 {
            if let Some(_) = board.color_on(&(x as u8, y as u8)) {
                nodes2.extend(script_str_compiled.generate_moves(&mut board, &(x, y), true).unwrap());
            }
        }
    }
    let duration3 = start3.elapsed();
    println!("{}, {}", nodes.len(), nodes2.len());
    println!("실행 소요 시간 (JIT): {:?}", duration2);
    println!("실행 소요 시간 (VMI): {:?}", duration3);

    for y in 0..TEST_BOARD_SIZE {
        for x in 0..TEST_BOARD_SIZE {
            if nodes.iter().any(|node| node.get_dest() == (x as u8, y as u8)) {
                print!("* ");
            }
            else if let Some(p) = board.piece_on(&(x as u8, y as u8)) {
                print!("{} ", &p[..1]);
            }
            else {
                print!(". ");
            }
        }
        println!();
    }

    println!("구동 완료.");
}