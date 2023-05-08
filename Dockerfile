FROM rust:latest

EXPOSE 8000

ARG CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

COPY . .

# Update the file date
RUN touch src/main.rs

RUN cargo build --release

CMD ["./target/release/hbp"]
