use axum::{
    Router, http::{HeaderMap, StatusCode}, response::{IntoResponse, Json}, routing::{get, post},
    extract::Json as JsonBody,
};
use chessembly_bot::{
    chessembly::{self, ChessemblyCompiled, Piece, PieceSpan, board::{Board, BoardState, BothBoardState}},
    engine,
};
use std::{collections::HashMap, env};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

// ── /apply 엔드포인트 입출력 타입 ─────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct ApplyMoveRequest {
    from: (u8, u8),
    move_to: (u8, u8),
    transition: Option<String>,
}

#[derive(serde::Serialize)]
struct BoardStateResponse {
    position: String,
    turn: String,
    castling_oo: String,
    castling_ooo: String,
    en_passant_white: String,
    en_passant_black: String,
}

fn encode_board_response<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(
    board: &Board<'a, MACHO, IMPRISONED, SIZE>,
) -> BoardStateResponse {
    let position = (0..SIZE)
        .map(|i| {
            (0..SIZE)
                .map(|j| match &board.board[i][j] {
                    PieceSpan::Piece(p) => format!(
                        "{}:{}",
                        p.piece_type,
                        if p.color == chessembly::Color::White { "white" } else { "black" }
                    ),
                    PieceSpan::Empty => ".".to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("/");

    let encode_ep = |ep: &Vec<chessembly::Position>| {
        if ep.is_empty() {
            ".".to_string()
        } else {
            ep.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join("/")
        }
    };

    BoardStateResponse {
        position,
        turn: if board.turn == chessembly::Color::White { "white".to_string() } else { "black".to_string() },
        castling_oo: format!(
            "{}{}",
            if board.board_state.white.castling_oo { '1' } else { '0' },
            if board.board_state.black.castling_oo { '1' } else { '0' },
        ),
        castling_ooo: format!(
            "{}{}",
            if board.board_state.white.castling_ooo { '1' } else { '0' },
            if board.board_state.black.castling_ooo { '1' } else { '0' },
        ),
        en_passant_white: encode_ep(&board.board_state.white.enpassant),
        en_passant_black: encode_ep(&board.board_state.black.enpassant),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(serve_debug_ui).post(run_engine))
        .route("/debug", post(run_engine_debug))
        .route("/moves", post(get_piece_moves))
        .route("/apply", post(apply_move_endpoint))
        .route("/classify", post(classify_piece))
        .route("/classifier", get(serve_classifier_ui))
        .layer(cors);

    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080); // PORT 환경 변수를 읽고, 없으면 8080 사용

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_debug_ui() -> impl IntoResponse {
    axum::response::Html(include_str!("debug.html"))
}

async fn serve_classifier_ui() -> impl IntoResponse {
    axum::response::Html(include_str!("piece_classifier.html"))
}

#[derive(Debug)]
struct SetupBoardParams<'a> {
    compiled: &'a ChessemblyCompiled<'a>,
    position: &'a str,
    board_state: BothBoardState<'a>,
    turn: chessembly::Color
}

fn setup_board<'a, const MACHO: bool, const IMPRISONED: bool, const SIZE: usize>(
    params: SetupBoardParams<'a>
) -> Board<'a, MACHO, IMPRISONED, SIZE> {
    let mut board = Board::<'a, MACHO, IMPRISONED, SIZE>::empty(&params.compiled);
    let mut i = 0;
    for line in params.position.split('/') {
        let mut j = 0;
        for pc in line.split_whitespace() {
            if let Some((piece_name, color)) = pc.split_once(':') {
                board.board[i][j] = chessembly::PieceSpan::Piece(chessembly::Piece {
                    piece_type: piece_name,
                    color: if color == "white" {
                        chessembly::Color::White
                    } else {
                        chessembly::Color::Black
                    },
                });
            }
            j += 1;
        }
        i += 1;
    }

    board.board_state = params.board_state;
    board.turn = params.turn;

    board
}

async fn run_engine(headers: HeaderMap) -> impl IntoResponse {
    let (
        Some(position),
        Some(script),
        Some(turn),
        Some(castling_oo),
        Some(castling_ooo),
        Some(en_passant_white),
        Some(en_passant_black),
        Some(register_white),
        Some(register_black),
        Some(depth_header_str),
    ) = (
        headers.get("Position"),
        headers.get("Chessembly"),
        headers.get("Turn"),
        headers.get("Castling-OO"),
        headers.get("Castling-OOO"),
        headers.get("En-Passant-White"),
        headers.get("En-Passant-Black"),
        headers.get("Register-White"),
        headers.get("Register-Black"),
        headers.get("Depth"),
    ) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    let board_size = headers.get("Board-Size").map(|b| b.to_str().map(|x| x.parse::<u8>().unwrap_or(8)).unwrap_or(8)).unwrap_or(8);
    println!("{:?}", board_size);
    
    let Ok(depth) = depth_header_str.to_str().map(|x| x.parse::<u8>().unwrap_or(3)) else {
        return (StatusCode::OK, "asdf").into_response();
    };
    
    if depth <= 1 || depth > engine::search::HARD_DEPTH {
        return (StatusCode::OK, "asdf").into_response();
    }
    
    let Ok(str_script) = script.to_str().map(|x| urlencoding::decode(x).expect("UTF-8")) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    let str_script_fixed = str_script.replace('{', " { ").replace('}', " } ");

    let Ok(compiled) = ChessemblyCompiled::from_script(&str_script_fixed[..]) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    let (
        Ok(castling_oo_tuple),
        Ok(castling_ooo_tuple),
        Ok(en_passant_white_str),
        Ok(en_passant_black_str),
        Ok(register_white_str),
        Ok(register_black_str)
    ) = (
        castling_oo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        castling_ooo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        en_passant_white.to_str(),
        en_passant_black.to_str(),
        register_white.to_str(),
        register_black.to_str()
    ) else {
        return (StatusCode::OK, "asdf").into_response();
    };
    let mut en_passant_white_positions: Vec<chessembly::Position> = Vec::new();
    let mut en_passant_black_positions: Vec<chessembly::Position> = Vec::new();
    let mut register_white_map: HashMap<&str, u8> = HashMap::new();
    let mut register_black_map: HashMap<&str, u8> = HashMap::new();
    for coord in en_passant_white_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_white_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for coord in en_passant_black_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_black_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for register in register_white_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_white_map.insert(key, value.parse().unwrap_or(0));
        }
    }
    for register in register_black_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_black_map.insert(key, value.parse().unwrap_or(0));
        }
    }

    let board_state_white = BoardState {
        castling_oo: castling_oo_tuple.0,
        castling_ooo: castling_ooo_tuple.0,
        enpassant: en_passant_white_positions,
        register: register_white_map
    };

    let board_state_black = BoardState {
        castling_oo: castling_oo_tuple.1,
        castling_ooo: castling_ooo_tuple.1,
        enpassant: en_passant_black_positions,
        register: register_black_map
    };

    let board_state = BothBoardState {
        white: board_state_white,
        black: board_state_black,
    };

    let turn = if turn.to_str().unwrap() == "white" {
        chessembly::Color::White
    } else {
        chessembly::Color::Black
    };
    
    let is_macho = headers.get("Macho").is_some();
    let is_imprisoned = headers.get("Imprisoned").is_some();
    let beam_width: Option<usize> = headers.get("Beam-Width")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(to_evaluate) = headers.get("Target") {
        let Ok(to_evaluate_str) = to_evaluate.to_str() else {
            return (StatusCode::OK, "asdf").into_response();
        };
        let Some((from_str, position_str)) = to_evaluate_str.split_once('/') else {
            return (StatusCode::OK, "asdf").into_response();
        };
        let Some(from) = from_str.split_once(',').map(|(x, y)| (x.parse().unwrap_or(0), y.parse().unwrap_or(0))) else {
            return (StatusCode::OK, "asdf").into_response();
        };
        let Some(position) = position_str.split_once(',').map(|(x, y)| (x.parse().unwrap_or(0), y.parse().unwrap_or(0))) else {
            return (StatusCode::OK, "asdf").into_response();
        };
        println!("{:?}/{:?}", from, position);
        return (StatusCode::OK, format!("{:?}/{:?}", from, position)).into_response();
    }

    let param = SetupBoardParams {
        compiled: &compiled,
        position: position.to_str().unwrap(),
        board_state: board_state,
        turn: turn
    };

    let best_move = match (is_macho, is_imprisoned, board_size) {
        (true, true, 9) => {
            let mut board: Board<true, false, 9> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        },
        (true, false, 9)  => {
            let mut board: Board<true, false, 9> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        (false, true, 9) => {
            let mut board: Board<false, true, 9> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        (false, false, 9) => {
            let mut board: Board<false, false, 9> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        
        (true, true, 8) | (true, true, _) => {
            let mut board: Board<true, false, 8> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        },
        (true, false, 8) | (true, false, _)  => {
            let mut board: Board<true, false, 8> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        (false, true, 8) | (false, true, _) => {
            let mut board: Board<false, true, 8> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        (false, false, 8) | (false, false, _) => {
            let mut board: Board<false, false, 8> = setup_board(param);
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
    };

    
    // if let Ok((node, score)) = best_move {
    //     println!("{:?}", node);
    //     return (StatusCode::OK, Json(BestMoveResponse { chess_move: node, score })).into_response();
    // } else if let Err(_) = best_move {
    //     return (StatusCode::OK, "null").into_response();
    // }
    // return (StatusCode::OK, "asdf").into_response();
    if let Ok(node) = best_move {
        return (StatusCode::OK, Json(node)).into_response();
    } else if let Err(_) = best_move {
        return (StatusCode::OK, "null").into_response();
    }
    return (StatusCode::OK, "asdf").into_response();
}

// ─── POST /debug — 디버그 모드 최선 수 계산 ──────────────────────────────────
// 기존 run_engine 과 동일한 헤더를 받지만, engine::search::find_best_move_debug 를
// 호출해 탐색 통계(nodes, qnodes, TT hit rate 등)를 포함한 JSON 을 반환합니다.
async fn run_engine_debug(headers: HeaderMap) -> impl IntoResponse {
    let (
        Some(position),
        Some(script),
        Some(turn),
        Some(castling_oo),
        Some(castling_ooo),
        Some(en_passant_white),
        Some(en_passant_black),
        Some(register_white),
        Some(register_black),
        Some(depth_header_str),
    ) = (
        headers.get("Position"),
        headers.get("Chessembly"),
        headers.get("Turn"),
        headers.get("Castling-OO"),
        headers.get("Castling-OOO"),
        headers.get("En-Passant-White"),
        headers.get("En-Passant-Black"),
        headers.get("Register-White"),
        headers.get("Register-Black"),
        headers.get("Depth"),
    ) else {
        return (StatusCode::BAD_REQUEST, "missing headers").into_response();
    };

    let Ok(depth) = depth_header_str.to_str().map(|x| x.parse::<u8>().unwrap_or(3)) else {
        return (StatusCode::BAD_REQUEST, "bad depth").into_response();
    };

    if depth <= 1 || depth > engine::search::HARD_DEPTH {
        return (StatusCode::BAD_REQUEST, "depth out of range").into_response();
    }

    let Ok(str_script) = script.to_str().map(|x| urlencoding::decode(x).expect("UTF-8")) else {
        return (StatusCode::BAD_REQUEST, "bad script").into_response();
    };
    let str_script_fixed = str_script.replace('{', " { ").replace('}', " } ");
    let Ok(compiled) = chessembly::ChessemblyCompiled::from_script(&str_script_fixed[..]) else {
        return (StatusCode::BAD_REQUEST, "script compile failed").into_response();
    };

    let (
        Ok(castling_oo_tuple),
        Ok(castling_ooo_tuple),
        Ok(en_passant_white_str),
        Ok(en_passant_black_str),
        Ok(register_white_str),
        Ok(register_black_str),
    ) = (
        castling_oo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        castling_ooo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        en_passant_white.to_str(),
        en_passant_black.to_str(),
        register_white.to_str(),
        register_black.to_str(),
    ) else {
        return (StatusCode::BAD_REQUEST, "bad headers").into_response();
    };

    let mut en_passant_white_positions: Vec<chessembly::Position> = Vec::new();
    let mut en_passant_black_positions: Vec<chessembly::Position> = Vec::new();
    let mut register_white_map: HashMap<&str, u8> = HashMap::new();
    let mut register_black_map: HashMap<&str, u8> = HashMap::new();
    for coord in en_passant_white_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_white_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for coord in en_passant_black_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_black_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for register in register_white_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_white_map.insert(key, value.parse().unwrap_or(0));
        }
    }
    for register in register_black_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_black_map.insert(key, value.parse().unwrap_or(0));
        }
    }

    let board_state = chessembly::board::BothBoardState {
        white: chessembly::board::BoardState {
            castling_oo: castling_oo_tuple.0,
            castling_ooo: castling_ooo_tuple.0,
            enpassant: en_passant_white_positions,
            register: register_white_map,
        },
        black: chessembly::board::BoardState {
            castling_oo: castling_oo_tuple.1,
            castling_ooo: castling_ooo_tuple.1,
            enpassant: en_passant_black_positions,
            register: register_black_map,
        },
    };

    let turn = if turn.to_str().unwrap_or("white") == "white" {
        chessembly::Color::White
    } else {
        chessembly::Color::Black
    };

    let is_macho = headers.get("Macho").is_some();
    let is_imprisoned = headers.get("Imprisoned").is_some();
    let beam_width: Option<usize> = headers.get("Beam-Width")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());
    let pos_str = position.to_str().unwrap_or("");

    let param = SetupBoardParams {
        compiled: &compiled,
        position: pos_str,
        board_state: board_state,
        turn: turn
    };

    let debug_info = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut board: chessembly::board::Board<true, true, 8> = setup_board(param);
            engine::search::find_best_move_debug(&mut board, depth, beam_width)
        }
        (true, false) => {
            let mut board: chessembly::board::Board<true, false, 8> = setup_board(param);
            engine::search::find_best_move_debug(&mut board, depth, beam_width)
        }
        (false, true) => {
            let mut board: chessembly::board::Board<false, true, 8> = setup_board(param);
            engine::search::find_best_move_debug(&mut board, depth, beam_width)
        }
        (false, false) => {
            let mut board: chessembly::board::Board<false, false, 8> = setup_board(param);
            engine::search::find_best_move_debug(&mut board, depth, beam_width)
        }
    };

    (StatusCode::OK, Json(debug_info)).into_response()
}

// ─── 새 엔드포인트: POST /moves ───────────────────────────────────────────────
// 헤더: Position, Chessembly, Turn, Castling-OO, Castling-OOO,
//       En-Passant-White, En-Passant-Black, Register-White, Register-Black,
//       Target (col,row)  — Macho / Imprisoned 옵션
// 반환: 해당 칸 기물의 합법적인 수 목록 (JSON 배열)
async fn get_piece_moves(headers: HeaderMap) -> impl IntoResponse {
    let (
        Some(position),
        Some(script),
        Some(turn),
        Some(castling_oo),
        Some(castling_ooo),
        Some(en_passant_white),
        Some(en_passant_black),
        Some(register_white),
        Some(register_black),
        Some(target_header),
    ) = (
        headers.get("Position"),
        headers.get("Chessembly"),
        headers.get("Turn"),
        headers.get("Castling-OO"),
        headers.get("Castling-OOO"),
        headers.get("En-Passant-White"),
        headers.get("En-Passant-Black"),
        headers.get("Register-White"),
        headers.get("Register-Black"),
        headers.get("Target"),
    ) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    // Target: "col,row"
    let Ok(target_str) = target_header.to_str() else {
        return (StatusCode::OK, "asdf").into_response();
    };
    let Some((col_str, row_str)) = target_str.split_once(',') else {
        return (StatusCode::OK, "asdf").into_response();
    };
    let target_col: u8 = col_str.trim().parse().unwrap_or(0);
    let target_row: u8 = row_str.trim().parse().unwrap_or(0);

    let Ok(str_script) = script.to_str().map(|x| urlencoding::decode(x).expect("UTF-8")) else {
        return (StatusCode::OK, "asdf").into_response();
    };
    let str_script_fixed = str_script.replace('{', " { ").replace('}', " } ");
    let Ok(compiled) = ChessemblyCompiled::from_script(&str_script_fixed[..]) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    let (
        Ok(castling_oo_tuple),
        Ok(castling_ooo_tuple),
        Ok(en_passant_white_str),
        Ok(en_passant_black_str),
        Ok(register_white_str),
        Ok(register_black_str),
    ) = (
        castling_oo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        castling_ooo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        en_passant_white.to_str(),
        en_passant_black.to_str(),
        register_white.to_str(),
        register_black.to_str(),
    ) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    let mut en_passant_white_positions: Vec<chessembly::Position> = Vec::new();
    let mut en_passant_black_positions: Vec<chessembly::Position> = Vec::new();
    let mut register_white_map: HashMap<&str, u8> = HashMap::new();
    let mut register_black_map: HashMap<&str, u8> = HashMap::new();
    for coord in en_passant_white_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_white_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for coord in en_passant_black_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_black_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for register in register_white_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_white_map.insert(key, value.parse().unwrap_or(0));
        }
    }
    for register in register_black_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_black_map.insert(key, value.parse().unwrap_or(0));
        }
    }

    let board_state = BothBoardState {
        white: BoardState {
            castling_oo: castling_oo_tuple.0,
            castling_ooo: castling_ooo_tuple.0,
            enpassant: en_passant_white_positions,
            register: register_white_map,
        },
        black: BoardState {
            castling_oo: castling_oo_tuple.1,
            castling_ooo: castling_ooo_tuple.1,
            enpassant: en_passant_black_positions,
            register: register_black_map,
        },
    };

    let turn = if turn.to_str().unwrap_or("white") == "white" {
        chessembly::Color::White
    } else {
        chessembly::Color::Black
    };

    let is_macho = headers.get("Macho").is_some();
    let is_imprisoned = headers.get("Imprisoned").is_some();

    let pos_str = position.to_str().unwrap_or("");

    let param = SetupBoardParams {
        compiled: &compiled,
        position: pos_str,
        board_state: board_state,
        turn: turn
    };

    let moves = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut b: Board<true, true, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<true, true, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<true, true, 8>(raw, &b)
        }
        (true, false) => {
            let mut b: Board<true, false, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<true, false, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<true, false, 8>(raw, &b)
        }
        (false, true) => {
            let mut b: Board<false, true, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<false, true, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<false, true, 8>(raw, &b)
        }
        (false, false) => {
            let mut b: Board<false, false, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<false, false, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<false, false, 8>(raw, &b)
        }
    };

    (StatusCode::OK, Json(moves)).into_response()
}

// ─── POST /apply ──────────────────────────────────────────────────────────────
// 현재 보드 상태 헤더 + JSON 바디 { from, move_to, transition? }
// → 해당 수를 서버에서 적용하고 새 보드 상태를 JSON으로 반환
async fn apply_move_endpoint(
    headers: HeaderMap,
    JsonBody(body): JsonBody<ApplyMoveRequest>,
) -> impl IntoResponse {
    let (
        Some(position),
        Some(script),
        Some(turn),
        Some(castling_oo),
        Some(castling_ooo),
        Some(en_passant_white),
        Some(en_passant_black),
        Some(register_white),
        Some(register_black),
    ) = (
        headers.get("Position"),
        headers.get("Chessembly"),
        headers.get("Turn"),
        headers.get("Castling-OO"),
        headers.get("Castling-OOO"),
        headers.get("En-Passant-White"),
        headers.get("En-Passant-Black"),
        headers.get("Register-White"),
        headers.get("Register-Black"),
    ) else {
        return (StatusCode::BAD_REQUEST, "missing headers").into_response();
    };

    let Ok(str_script) = script.to_str().map(|x| urlencoding::decode(x).expect("UTF-8")) else {
        return (StatusCode::BAD_REQUEST, "bad script").into_response();
    };
    let str_script_fixed = str_script.replace('{', " { ").replace('}', " } ");
    let Ok(compiled) = ChessemblyCompiled::from_script(&str_script_fixed[..]) else {
        return (StatusCode::BAD_REQUEST, "script compile failed").into_response();
    };

    let (
        Ok(castling_oo_tuple),
        Ok(castling_ooo_tuple),
        Ok(en_passant_white_str),
        Ok(en_passant_black_str),
        Ok(register_white_str),
        Ok(register_black_str),
    ) = (
        castling_oo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        castling_ooo.to_str().map(|x| (x.chars().nth(0) == Some('1'), x.chars().nth(1) == Some('1'))),
        en_passant_white.to_str(),
        en_passant_black.to_str(),
        register_white.to_str(),
        register_black.to_str(),
    ) else {
        return (StatusCode::BAD_REQUEST, "bad headers").into_response();
    };

    let mut en_passant_white_positions: Vec<chessembly::Position> = Vec::new();
    let mut en_passant_black_positions: Vec<chessembly::Position> = Vec::new();
    let mut register_white_map: HashMap<&str, u8> = HashMap::new();
    let mut register_black_map: HashMap<&str, u8> = HashMap::new();
    for coord in en_passant_white_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_white_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for coord in en_passant_black_str.split('/') {
        if let Some((x, y)) = coord.split_once(',') {
            en_passant_black_positions.push((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }
    for register in register_white_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_white_map.insert(key, value.parse().unwrap_or(0));
        }
    }
    for register in register_black_str.split('/') {
        if let Some((key, value)) = register.split_once(',') {
            register_black_map.insert(key, value.parse().unwrap_or(0));
        }
    }

    let board_state = BothBoardState {
        white: BoardState {
            castling_oo: castling_oo_tuple.0,
            castling_ooo: castling_ooo_tuple.0,
            enpassant: en_passant_white_positions,
            register: register_white_map,
        },
        black: BoardState {
            castling_oo: castling_oo_tuple.1,
            castling_ooo: castling_ooo_tuple.1,
            enpassant: en_passant_black_positions,
            register: register_black_map,
        },
    };

    let turn = if turn.to_str().unwrap_or("white") == "white" {
        chessembly::Color::White
    } else {
        chessembly::Color::Black
    };

    let is_macho = headers.get("Macho").is_some();
    let is_imprisoned = headers.get("Imprisoned").is_some();
    let pos_str = position.to_str().unwrap_or("");

    let param = SetupBoardParams {
        compiled: &compiled,
        position: pos_str,
        board_state: board_state,
        turn: turn
    };

    // 합법적인 수 목록에서 요청된 수를 찾아 적용
    let result: Option<BoardStateResponse> = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut b: Board<true, true, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<true, true, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<true, true, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
        (true, false) => {
            let mut b: Board<true, false, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<true, false, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<true, false, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
        (false, true) => {
            let mut b: Board<false, true, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<false, true, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<false, true, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
        (false, false) => {
            let mut b: Board<false, false, 8> = setup_board(param);
            let script = b.script;
            let raw = script.get_moves::<false, false, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<false, false, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
    };

    match result {
        Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
        None => (StatusCode::BAD_REQUEST, "illegal move").into_response(),
    }
}

// ─── POST /classify ───────────────────────────────────────────────────────────
// 바디: { "piece_name": "...", "script": "..." }
// 반환: { "classification": "legend"|"major"|"minor", "example": "..." | null }
//
// 분류 기준:
//   legend — 자신 킹 없이 이 기물 혼자서 체크메이트 가능한 포지션이 존재
//   major  — 자신 킹과 함께 체크메이트 가능한 포지션이 존재
//   minor  — 위 두 경우 모두 해당 없음

#[derive(serde::Deserialize)]
struct ClassifyRequest {
    piece_name: String,
    script: String,
}

#[derive(serde::Serialize)]
struct ClassifyResponse {
    classification: String,
    example: Option<String>,
}

async fn classify_piece(JsonBody(body): JsonBody<ClassifyRequest>) -> impl IntoResponse {
    // piece_name을 스크립트 주석으로 포함시켜 같은 lifetime 'a를 공유하게 함
    // ';'로 주석 체인을 분리해야 from_script가 첫 번째 실제 체인을 건너뛰지 않음
    let combined = format!(
        "#{};\n{}",
        body.piece_name,
        body.script.replace('{', " { ").replace('}', " } ")
    );
    let Ok(compiled) = ChessemblyCompiled::from_script(&combined) else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "script parse failed"}))).into_response();
    };
    // combined[1..] 은 '#' 이후의 piece_name 부분을 포함하므로 lifetime이 동일
    let piece_name: &str = &combined[1..1 + body.piece_name.len()];

    // ── Legend 단계: 어떤 위치에서든 이 기물의 이동 가능 칸이 2×2 블록을 커버하면 legend ──
    // 빈 보드에서 검사해야 슬라이딩 기물(비숍·퀸 계열)의 이동 범위가 완전히 펼쳐짐
    // 꽉 찬 보드를 쓰면 슬라이딩이 1칸으로 막혀 아마존·아치비숍 등이 2×2를 커버하지 못함
    for pc in 0u8..8 {
        for pr in 0u8..8 {
            // 백 킹은 (7,7) 고정 (기물과 겹치면 (7,6) 사용)
            let (wkc, wkr): (u8, u8) = if (pc, pr) != (7, 7) { (7, 7) } else { (7, 6) };

            let mut board = Board::<false, false, 8>::empty(&compiled);
            board.board_state.white.castling_oo = false;
            board.board_state.white.castling_ooo = false;
            board.board_state.black.castling_oo = false;
            board.board_state.black.castling_ooo = false;
            board.board[pr as usize][pc as usize] = PieceSpan::Piece(Piece { piece_type: piece_name, color: chessembly::Color::White });
            board.board[wkr as usize][wkc as usize] = PieceSpan::Piece(Piece { piece_type: "king", color: chessembly::Color::White });

            let script = board.script;
            let moves = script.get_moves::<false, false, 8>(&mut board, &(pc, pr), true);
            let dests: Vec<(u8, u8)> = moves.iter().map(|m| m.get_dest()).collect();

            // 이동 가능 칸들이 2×2 블록을 커버하는지 확인
            for c in 0u8..7 {
                for r in 0u8..7 {
                    if dests.contains(&(c, r)) && dests.contains(&(c + 1, r))
                        && dests.contains(&(c, r + 1)) && dests.contains(&(c + 1, r + 1))
                    {
                        return (StatusCode::OK, Json(ClassifyResponse {
                            classification: "legend".to_string(),
                            example: Some(format!(
                                "{}@({},{}), 2x2 cover ({},{})~({},{})",
                                piece_name, pc, pr, c, r, c + 1, r + 1
                            )),
                        })).into_response();
                    }
                }
            }
        }
    }

    // ── Major 단계: 행 또는 열 방향으로 연속된 두 칸에 이동 가능하면 major ─────────────
    // 빈 보드에서 슬라이딩 범위를 확인
    for pc in 0u8..8 {
        for pr in 0u8..8 {
            let (wkc, wkr): (u8, u8) = if (pc, pr) != (7, 7) { (7, 7) } else { (7, 6) };

            let mut board = Board::<false, false, 8>::empty(&compiled);
            board.board_state.white.castling_oo = false;
            board.board_state.white.castling_ooo = false;
            board.board_state.black.castling_oo = false;
            board.board_state.black.castling_ooo = false;
            board.board[pr as usize][pc as usize] = PieceSpan::Piece(Piece { piece_type: piece_name, color: chessembly::Color::White });
            board.board[wkr as usize][wkc as usize] = PieceSpan::Piece(Piece { piece_type: "king", color: chessembly::Color::White });

            let script = board.script;
            let moves = script.get_moves::<false, false, 8>(&mut board, &(pc, pr), true);
            let dests: Vec<(u8, u8)> = moves.iter().map(|m| m.get_dest()).collect();

            // 같은 행에서 인접한 두 칸에 이동 가능하면 행 방향 슬라이딩
            for r in 0u8..8 {
                for c in 0u8..7 {
                    if dests.contains(&(c, r)) && dests.contains(&(c + 1, r)) {
                        return (StatusCode::OK, Json(ClassifyResponse {
                            classification: "major".to_string(),
                            example: Some(format!(
                                "{}@({},{}), horizontal slide row {}",
                                piece_name, pc, pr, r
                            )),
                        })).into_response();
                    }
                }
            }
            // 같은 열에서 인접한 두 칸에 이동 가능하면 열 방향 슬라이딩
            for c in 0u8..8 {
                for r in 0u8..7 {
                    if dests.contains(&(c, r)) && dests.contains(&(c, r + 1)) {
                        return (StatusCode::OK, Json(ClassifyResponse {
                            classification: "major".to_string(),
                            example: Some(format!(
                                "{}@({},{}), vertical slide col {}",
                                piece_name, pc, pr, c
                            )),
                        })).into_response();
                    }
                }
            }
        }
    }

    (StatusCode::OK, Json(ClassifyResponse {
        classification: "minor".to_string(),
        example: None,
    })).into_response()
}
