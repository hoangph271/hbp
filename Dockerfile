FROM rust:latest
COPY . .

EXPOSE 8000

RUN cargo build --release
CMD ["./target/release/hbp"]
