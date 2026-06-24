FROM rust:1.96

WORKDIR /app

COPY . .

RUN cargo build --release -p aruna-node

CMD ["./target/release/aruna-node"]
