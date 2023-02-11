FROM rust:latest
WORKDIR /usr/src/hbp
COPY . .

RUN rustup component add rustfmt

RUN cargo install --path .

EXPOSE 8000

CMD ["hbp"]
