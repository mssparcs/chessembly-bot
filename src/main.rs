use axum::{
    Router, http::{HeaderMap, StatusCode}, response::{IntoResponse, Json}, routing::{get, post},
    extract::Json as JsonBody,
};
use chessembly_bot::{
    chessembly::{self, ChessemblyCompiled, PieceSpan, board::{Board, BoardState, BothBoardState}},
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
        .route("/moves", post(get_piece_moves))
        .route("/apply", post(apply_move_endpoint))
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

fn setup_board<'a, const MACHO: bool, const IMPRISONED: bool>(
    compiled: &'a ChessemblyCompiled<'a>,
    position: &'a str,
    board_state: BothBoardState<'a>,
    turn: chessembly::Color,
) -> Board<'a, MACHO, IMPRISONED, 8> {
    let mut board = Board::<'a, MACHO, IMPRISONED, 8>::empty(&compiled);
    let mut i = 0;
    for line in position.split('/') {
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

    board.board_state = board_state;
    board.turn = turn;

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
    
    let Ok(depth) = depth_header_str.to_str().map(|x| x.parse::<u8>().unwrap_or(3)) else {
        return (StatusCode::OK, "asdf").into_response();
    };
    
    if depth <= 1 || depth > 4 {
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

    let best_move = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut board: Board<true, true, 8> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth, beam_width)
        },
        (true, false) => {
            let mut board: Board<true, false, 8> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        (false, true) => {
            let mut board: Board<false, true, 8> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth, beam_width)
        }
        (false, false) => {
            let mut board: Board<false, false, 8> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
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

    let moves = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut b: Board<true, true, 8> = setup_board(&compiled, pos_str, board_state, turn);
            let script = b.script;
            let raw = script.get_moves::<true, true, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<true, true, 8>(raw, &b)
        }
        (true, false) => {
            let mut b: Board<true, false, 8> = setup_board(&compiled, pos_str, board_state, turn);
            let script = b.script;
            let raw = script.get_moves::<true, false, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<true, false, 8>(raw, &b)
        }
        (false, true) => {
            let mut b: Board<false, true, 8> = setup_board(&compiled, pos_str, board_state, turn);
            let script = b.script;
            let raw = script.get_moves::<false, true, 8>(&mut b, &(target_col, target_row), true);
            script.filter_nodes::<false, true, 8>(raw, &b)
        }
        (false, false) => {
            let mut b: Board<false, false, 8> = setup_board(&compiled, pos_str, board_state, turn);
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

    // 합법적인 수 목록에서 요청된 수를 찾아 적용
    let result: Option<BoardStateResponse> = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut b: Board<true, true, 8> = setup_board(&compiled, pos_str, board_state, turn);
            let script = b.script;
            let raw = script.get_moves::<true, true, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<true, true, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
        (true, false) => {
            let mut b: Board<true, false, 8> = setup_board(&compiled, pos_str, board_state, turn);
            let script = b.script;
            let raw = script.get_moves::<true, false, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<true, false, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
        (false, true) => {
            let mut b: Board<false, true, 8> = setup_board(&compiled, pos_str, board_state, turn);
            let script = b.script;
            let raw = script.get_moves::<false, true, 8>(&mut b, &body.from, true);
            let filtered = script.filter_nodes::<false, true, 8>(raw, &b);
            filtered.into_iter()
                .find(|m| m.get_dest() == body.move_to && m.get_promotion().as_deref() == body.transition.as_deref())
                .map(|m| encode_board_response(&b.make_move_new(&m)))
        }
        (false, false) => {
            let mut b: Board<false, false, 8> = setup_board(&compiled, pos_str, board_state, turn);
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
