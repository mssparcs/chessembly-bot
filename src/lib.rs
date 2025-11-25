use worker::*;

use crate::chessembly::{Board, ChessemblyCompiled};

pub mod chessembly;
pub mod engine;

// fn router() -> Router {
//     Router::new().route("/", get(root))
// }

#[event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, _ctx: Context) -> Result<worker::Response> {
    // return Ok(worker::Response::from_body(ResponseBody::Body(String::from("null").into_bytes())).unwrap());
    let (Some(position), Some(script), Some(data)) = (
        req.headers().get("position"),
        req.headers().get("Chessembly"),
        req.headers().get("Turn"),
    ) else {
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
    };

    let Ok(str_script) = script.to_str() else {
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
    };

    let Ok(compiled) = ChessemblyCompiled::from_script(str_script) else {
        return Ok(Response::from_body(ResponseBody::Body(
            "asdf".as_bytes().to_vec(),
        ))?);
    };

    console_log!("{:?}", compiled.chains);
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
    board.turn = if data.to_str().unwrap() == "white" {
        chessembly::Color::White
    } else {
        chessembly::Color::Black
    };

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
