FROM rust:1
COPY . .

EXPOSE 8000

RUN cargo build --release
CMD ["./target/release/hbp"]
