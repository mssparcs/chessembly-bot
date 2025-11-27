# 공식 Rust 이미지를 사용합니다.
FROM rust:1.73.0

WORKDIR /usr/src/app
COPY . .

# 'cargo install' 대신 'cargo build'를 사용하여 릴리즈 모드로 애플리케이션을 빌드합니다.
RUN cargo build --release

# Cloud Run은 PORT 환경 변수(기본값 8080)를 주입합니다.
# Axum 서버가 이 포트를 사용하도록 설정해야 합니다.
EXPOSE 8080

# 빌드된 결과물을 직접 실행합니다.
CMD ["./target/release/chessembly-bot"]
