FROM rust:latest
COPY . .

EXPOSE 8000

ARG CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN cargo build --release
CMD ["./target/release/hbp"]
