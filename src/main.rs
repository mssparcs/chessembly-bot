use axum::{
    Router, http::{HeaderMap, StatusCode}, response::{IntoResponse, Json}, routing::post
};
use chessembly_bot::{
    chessembly::{self, ChessemblyCompiled, board::{Board, BoardState, BothBoardState}},
    engine,
};
use std::{collections::HashMap, env};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let app = Router::new().route("/", post(run_engine));

    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080); // PORT 환경 변수를 읽고, 없으면 8080 사용

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn setup_board<'a, const MACHO: bool, const IMPRISONED: bool>(
    compiled: &'a ChessemblyCompiled<'a>,
    position: &'a str,
    board_state: BothBoardState<'a>,
    turn: chessembly::Color,
) -> Board<'a, MACHO, IMPRISONED> {
    let mut board = Board::<'a, MACHO, IMPRISONED>::empty(&compiled);
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
    
    if depth <= 0 || depth > 4 {
        return (StatusCode::OK, "asdf").into_response();
    }

    let Ok(str_script) = script.to_str().map(|x| urlencoding::decode(x).expect("UTF-8")) else {
        return (StatusCode::OK, "asdf").into_response();
    };

    let Ok(compiled) = ChessemblyCompiled::from_script(&str_script[..]) else {
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

    let best_move = match (is_macho, is_imprisoned) {
        (true, true) => {
            let mut board: Board<true, true> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth)
        },
        (true, false) => {
            let mut board: Board<true, false> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth)
        }
        (false, true) => {
            let mut board: Board<false, true> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth)
        }
        (false, false) => {
            let mut board: Board<false, false> = setup_board(
                &compiled,
                position.to_str().unwrap(),
                board_state,
                turn
            );
            engine::search::find_best_move(&mut board, depth)
        }
    };

    if let Ok(node) = best_move {
        return (StatusCode::OK, Json(node)).into_response();
    } else if let Err(_) = best_move {
        return (StatusCode::OK, "null").into_response();
    }
    return (StatusCode::OK, "asdf").into_response();
}
