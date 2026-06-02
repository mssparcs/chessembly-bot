mod helper;

use crate::chessembly::behavior::BehaviorChain;
use std::collections::HashMap;
use std::time::Instant;
use std::ffi::c_void;
use helper::*;
use std::mem;
use std::ptr;

use super::{
    DeltaPosition,
    WallCollision,
    ChessMove,
    PieceSpan,
    Behavior,
    MoveType,
    Position,
    Board,
    Color
};

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct ChessemblyCompiled<'a> {
//     _marker: std::marker::PhantomData<&'a ()>,
// }

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

    pub check_danger: bool,
    
    pub transition_ptr: *const u8,             // offset 1848 (Thin pointer로 변경하여 컴파일 에러 예방)
    pub transition_len: u64,                   // offset 1856
    
    pub state_change_keys: [*const u8; 32],    // offset 1864 (ends at 2120)
    pub state_change_key_lens: [u64; 32],      // offset 2120 (ends at 2376)
    pub state_change_vals: [u8; 32],           // offset 2376 (ends at 2408)
    pub state_change_len: u64,                 // offset 2408
}

impl<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize> JitContext<'a, MACHO, IMPRISONED, SIZE> {
    pub fn new(board: &Board<'a, MACHO, IMPRISONED, SIZE>, start_pos: Position, nodes: &mut Vec<ChessMove<'a>>, check_danger: bool) -> Self {
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
            check_danger,
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
            self.emit(&[0x48, 0xc7, 0xc6]);          // mov rsi, dx (imm32 sign-extends to 64)
            self.emit(&(dx as i32).to_le_bytes());
            self.emit(&[0x48, 0xc7, 0xc2]);          // mov rdx, dy (imm32 sign-extends to 64)
            self.emit(&(dy as i32).to_le_bytes());
            
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
            self.emit(&[0x49, 0xc7, 0xc0]);          // mov r8, name_len (imm32, fits)
            self.emit(&(name_len as u32).to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x28]);    // sub rsp, 40
        }
        #[cfg(unix)]
        {
            self.emit(&[0x48, 0x89, 0xdf]);          // mov rdi, rbx
            self.emit(&[0x48, 0xbe]);                // mov rsi, name_ptr (imm64)
            self.emit(&name_ptr.to_le_bytes());
            self.emit(&[0x48, 0xba]);                // mov rdx, name_len (imm64)
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

    fn emit_call_piece_on<'a>(&mut self, func_ptr: usize, name: &'a str, dx: i8, dy: i8) {
        let name_ptr = name.as_ptr() as u64;
        let name_len = name.len() as u64;
        
        // dx와 dy를 하나의 u64 파라미터 영역에 컴팩트하게 비트 패킹
        let packed_delta = ((dx as u32 & 0xFF) | (((dy as u32) & 0xFF) << 8)) as u64;

        #[cfg(target_os = "windows")]
        {
            self.emit(&[0x48, 0x89, 0xd9]);          // mov rcx, rbx (ctx)
            
            self.emit(&[0x48, 0xba]);                // mov rdx, name_ptr
            self.emit(&name_ptr.to_le_bytes());
            
            self.emit(&[0x49, 0xc7, 0xc0]);          // mov r8, name_len (imm32, len fits in 32 bits)
            self.emit(&(name_len as u32).to_le_bytes());

            self.emit(&[0x49, 0xc7, 0xc1]);          // mov r9, packed_delta (imm32, fits in 32 bits)
            self.emit(&(packed_delta as u32).to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x28]);    // sub rsp, 40 (Shadow Space + Alignment)
        }
        #[cfg(unix)]
        {
            self.emit(&[0x48, 0x89, 0xdf]);          // mov rdi, rbx (ctx)
            
            self.emit(&[0x48, 0xbe]);                // mov rsi, name_ptr (imm64)
            self.emit(&name_ptr.to_le_bytes());
            
            self.emit(&[0x48, 0xba]);                // mov rdx, name_len (imm64)
            self.emit(&name_len.to_le_bytes());

            self.emit(&[0x48, 0xb9]);                // mov rcx, packed_delta (imm64)
            self.emit(&packed_delta.to_le_bytes());
            
            self.emit(&[0x48, 0x83, 0xec, 0x08]);    // sub rsp, 8 (align 16-bytes)
        }

        self.emit(&[0x48, 0xb8]);                    // mov rax, func_ptr
        self.emit(&(func_ptr as u64).to_le_bytes());
        self.emit(&[0xff, 0xd0]);                    // call rax

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
                | Behavior::True
                | Behavior::False
                | Behavior::Write(_)
                | Behavior::Read(_)
                | Behavior::ReadAnd(_)
                | Behavior::ReadOr(_)
                | Behavior::ReadXor(_)

                | Behavior::BlockClose // ??
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
                Behavior::Move((dx, dy)) => self.emit_call_3_args(jit_helper_move::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Take((dx, dy)) => self.emit_call_3_args(jit_helper_take::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::TakeMove((dx, dy)) => self.emit_call_3_args(jit_helper_take_move::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Jump((dx, dy)) => self.emit_call_3_args(jit_helper_jump::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Catch((dx, dy)) => self.emit_call_3_args(jit_helper_catch::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Peek((dx, dy)) => self.emit_call_3_args(jit_helper_peek::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Observe((dx, dy)) => self.emit_call_3_args(jit_helper_observe::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                
                Behavior::Bound((dx, dy)) => self.emit_call_3_args(jit_helper_bound::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Edge((dx, dy)) => self.emit_call_3_args(jit_helper_edge::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::Corner((dx, dy)) => self.emit_call_3_args(jit_helper_corner::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),

                Behavior::EdgeTop((dx, dy)) => self.emit_call_3_args(jit_helper_edge_top::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::EdgeBottom((dx, dy)) => self.emit_call_3_args(jit_helper_edge_bottom::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::EdgeLeft((dx, dy)) => self.emit_call_3_args(jit_helper_edge_left::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::EdgeRight((dx, dy)) => self.emit_call_3_args(jit_helper_edge_right::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::CornerTopLeft((dx, dy)) => self.emit_call_3_args(jit_helper_corner_top_left::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::CornerTopRight((dx, dy)) => self.emit_call_3_args(jit_helper_corner_top_right::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::CornerBottomLeft((dx, dy)) => self.emit_call_3_args(jit_helper_corner_bottom_left::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                Behavior::CornerBottomRight((dx, dy)) => self.emit_call_3_args(jit_helper_corner_bottom_right::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy),
                
                Behavior::ColorOn((color_name, (dx, dy))) => {
                    if color_name == &"white" {
                        self.emit_call_3_args(jit_helper_color_on_white::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                    }
                    else {
                        self.emit_call_3_args(jit_helper_color_on_black::<MACHO, IMPRISONED, SIZE> as usize, *dx, *dy);
                    }
                }
                Behavior::Color(color_name) => {
                    if color_name == &"white" {
                        self.emit_call_native(jit_helper_color_white::<MACHO, IMPRISONED, SIZE> as usize);
                    }
                    else {
                        self.emit_call_native(jit_helper_color_black::<MACHO, IMPRISONED, SIZE> as usize);
                    }
                }
                Behavior::Piece(name) => {
                    self.emit_call_string_helper(jit_helper_piece::<MACHO, IMPRISONED, SIZE> as usize, name);
                }
                Behavior::PieceOn((name, (dx, dy))) => {
                    self.emit_call_piece_on(jit_helper_piece_on::<MACHO, IMPRISONED, SIZE> as usize, name, *dx, *dy);
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
                    self.emit_call_native(jit_helper_not::<MACHO, IMPRISONED, SIZE> as usize);
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
                if patch.target_label_id < 10010 && 10000 <= patch.target_label_id {
                    println!("{} {}", &patch.target_label_id - 10000, target_offset);
                }
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
#[derive(Debug, PartialEq, Eq)]
pub struct CompiledChain {
    ptr: *mut c_void,
    size: usize,
}

impl CompiledChain {
    pub unsafe fn execute<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, ctx: &mut JitContext<'a, MACHO, IMPRISONED, SIZE>) {
        let func: extern "C" fn(*mut JitContext<'a, MACHO, IMPRISONED, SIZE>) = mem::transmute(self.ptr);
        func(ctx);
    }

    pub fn execute_from<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(&self, board: &Board<'a, MACHO, IMPRISONED, SIZE>, start_pos: Position, nodes: &mut Vec<ChessMove<'a>>, check_danger: bool) {
        let mut ctx = JitContext::new(board, start_pos, nodes, check_danger);

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
    let compiled_chains = &script_str_compiled.compiled_chains;

    const TEST_BOARD_SIZE: usize = 8;

    let start = Instant::now();
    // let mut compiled_chains = Vec::new();

    // for chain in chains {
    //     let mut compiler = ChessemblyJitCompiler::new();
    //     let compiled = compiler.compile::<false, false, TEST_BOARD_SIZE>(chain);
    //     compiled_chains.push(compiled);
    // }
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
            for compiled in compiled_chains {
                if let Some(_) = board.color_on(&(x as u8, y as u8)) {
                    compiled.execute_from(&board, (x, y), &mut nodes, true);
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