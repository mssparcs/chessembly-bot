pub mod chessembly;
pub mod engine;

/*
use std::collections::HashMap;

use worker::*;

use crate::chessembly::{Board, ChessemblyCompiled, board::{BoardState, BothBoardState}};


// fn router() -> Router {
//     Router::new().route("/", get(root))
// }

#[event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, _ctx: Context) -> Result<worker::Response> {
    // return Ok(worker::Response::from_body(ResponseBody::Body(String::from("null").into_bytes())).unwrap());
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
        req.headers().get("Position"),
        req.headers().get("Chessembly"),
        req.headers().get("Turn"),
        req.headers().get("Castling-OO"),
        req.headers().get("Castling-OOO"),
        req.headers().get("En-Passant-White"),
        req.headers().get("En-Passant-Black"),
        req.headers().get("Register-White"),
        req.headers().get("Register-Black"),
    ) else {
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
    };

    let Ok(str_script) = worker::js_sys::decode_uri(script.to_str().unwrap()).map(|x| String::from(x)) else {
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
    };

    let Ok(compiled) = ChessemblyCompiled::from_script(&str_script[..]) else {
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
    };

    // console_log!("{:?}", compiled.chains);
    // console_log!("{:?}", castling_oo);
    // console_log!("{:?}", castling_ooo);
    // console_log!("{:?}", en_passant_white);
    // console_log!("{:?}", en_passant_black);
    // console_log!("{:?}", register_white);
    // console_log!("{:?}", register_black);

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
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
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

    let mut board = Board::empty(&compiled);
    let mut i = 0;
    for line in position.to_str().unwrap().split('/') {
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
    board.turn = if turn.to_str().unwrap() == "white" {
        chessembly::Color::White
    } else {
        chessembly::Color::Black
    };
    board.board_state = board_state;

    // worker::console_log!("{}", board.to_string());

    let best_move = engine::search::find_best_move(&mut board, 3);
    if let Ok(node) = best_move {
        return Ok(Response::from_json(&node)?);
    } else if let Err(n) = best_move {
        console_log!("????? {}", n);
        return Ok(Response::from_body(ResponseBody::Body(
            String::from("null").into_bytes(),
        ))?);
    }
    // println!("{:?}", req.body());
    Ok(Response::from_body(ResponseBody::Body(
        "asdf".as_bytes().to_vec(),
    ))?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
*/