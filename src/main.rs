use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use chessembly_bot::{
    chessembly::{self, board::Board, ChessemblyCompiled},
    engine
};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let app = Router::new().route("/", post(run_engine));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_engine(headers: HeaderMap) -> impl IntoResponse {
        let (Some(position), Some(script), Some(data)) = (
        headers.get("position"),
        headers.get("Chessembly"),
        headers.get("Turn"),
    ) else {
        return (StatusCode::OK, "asdf").into_response();
        // return Ok(Response::from_body(ResponseBody::Body(
        //     "asdf".as_bytes().to_vec(),
        // ))?);
    };

    let Ok(str_script) = script.to_str() else {
        return (StatusCode::OK, "asdf").into_response();
        // return Ok(Response::from_body(ResponseBody::Body(
        //     "asdf".as_bytes().to_vec(),
        // ))?);
    };

    let Ok(compiled) = ChessemblyCompiled::from_script(str_script) else {
        return (StatusCode::OK, "asdf").into_response();
        // return Ok(Response::from_body(ResponseBody::Body(
        //     "asdf".as_bytes().to_vec(),
        // ))?);
    };

    // console_log!("{:?}", compiled.chains);
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
        // return Ok(Response::from_json(&node)?);
        return (StatusCode::OK, Json(node)).into_response();
    } else if let Err(n) = best_move {
        // console_log!("????? {}", n);
        return (StatusCode::OK, "null").into_response();
    }
    // println!("{:?}", req.body());
    return (StatusCode::OK, "asdf").into_response();

    /*
    let (Some(position), Some(script), Some(data)) = (
        headers.get("position"),
        headers.get("Chessembly"),
        headers.get("Turn"),
    ) else {
        return (StatusCode::BAD_REQUEST, "Missing required headers").into_response();
    };

    let (Ok(str_script), Ok(position_str), Ok(turn_str)) = (
        script.to_str(),
        position.to_str(),
        data.to_str(),
    ) else {
        return (StatusCode::BAD_REQUEST, "Invalid header format").into_response();
    };

    let Ok(compiled) = ChessemblyCompiled::from_script(str_script) else {
        return (StatusCode::BAD_REQUEST, "Failed to compile Chessembly script").into_response();
    };

    let mut board = Board::empty(&compiled);
    for (i, line) in position_str.split('/').enumerate().take(board.board.len()) {
        for (j, pc) in line.split_whitespace().enumerate().take(board.board[i].len()) {
            if let Some((piece_name, color)) = pc.split_once(':') {
                board.board[i][j] = PieceSpan::Piece(Piece {
                    piece_type: piece_name,
                    color: if color == "white" {
                        chessembly_bot::chessembly::Color::White
                    } else {
                        chessembly_bot::chessembly::Color::Black
                    },
                });
            }
        }
    }

    board.turn = if turn_str == "white" {
        chessembly_bot::chessembly::Color::White
    } else {
        chessembly_bot::chessembly::Color::Black
    };

    match engine::search::find_best_move(&mut board, 3) {
        Ok(node) => (StatusCode::OK, Json(node)).into_response(),
        Err(_) => (StatusCode::OK, Json(Value::Null)).into_response(),
    }
    */
}
