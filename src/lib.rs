use worker::*;

use crate::chessembly::{Board, ChessemblyCompiled};

pub mod chessembly;
pub mod engine;

// fn router() -> Router {
//     Router::new().route("/", get(root))
// }

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<worker::Response> {

    // return Ok(worker::Response::from_body(ResponseBody::Body(String::from("null").into_bytes())).unwrap());

    if let Some(position) = req.headers().get("Position") {
        if let Some(script) = req.headers().get("Chessembly") {
            if let Some(data) = req.headers().get("Turn") {
                if let Ok(compiled) = ChessemblyCompiled::from_script(&String::from(worker::js_sys::decode_uri(script.to_str().unwrap()).unwrap())[..]) {
                    // worker::console_log!("{:?}", compiled.chains);
                    let mut board = Board::empty(&compiled);
                    let mut i = 0;
                    for line in position.to_str().unwrap().split('/') {
                        let mut j = 0;
                        for pc in line.split_whitespace() {
                            if let Some((piece_name, color)) = pc.split_once(':') {
                                board.board[i][j] = chessembly::PieceSpan::Piece(chessembly::Piece {
                                    piece_type: piece_name,
                                    color: if color == "white" { chessembly::Color::White } else { chessembly::Color::Black }
                                });
                            }
                            j += 1;
                        }
                        i += 1;
                    }
                    board.turn = if data.to_str().unwrap() == "white" { chessembly::Color::White } else { chessembly::Color::Black };

                    // worker::console_log!("{}", board.to_string());
                    
                    let best_move = engine::search::find_best_move(&mut board, 3);
                    if let Ok(node) = best_move {
                        return Ok(worker::Response::from_json(&node).unwrap());
                    }
                    else if let Err(n) = best_move {
                        worker::console_log!("????? {}", n);
                        return Ok(worker::Response::from_body(ResponseBody::Body(String::from("null").into_bytes())).unwrap());
                    }
                }
            }
        }
    }
    // println!("{:?}", req.body());
    Ok(worker::Response::from_body(ResponseBody::Body(String::from("asdf").into_bytes())).unwrap())
}

pub async fn root() -> &'static str {
    "Hello Axum!"

}