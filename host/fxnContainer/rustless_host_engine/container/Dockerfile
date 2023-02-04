FROM debian:bullseye

# Update default packages
RUN apt-get -qq update

# Get Ubuntu packages
RUN apt-get install -y -q \
    build-essential \
    curl \
    ca-certificates wget gcc libssl-dev libc6-dev pkg-config

# Get Rust; NOTE: using sh for better compatibility with other base images
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add .cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

COPY code /code

RUN cd /code && cargo build --release
WORKDIR /code

CMD ["cargo", "run", "--release", "--", "--port", "8080"]
EXPOSE 8080/tcp