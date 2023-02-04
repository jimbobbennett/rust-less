FROM debian:bullseye

# Update default packages
RUN apt-get -qq update

# Get Ubuntu packages
RUN apt-get install -y -q \
    build-essential \
    curl

# Get Rust; NOTE: using sh for better compatibility with other base images
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add .cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Check cargo is visible
RUN cargo --help

COPY rustless_host_engine /rustless_host_engine

RUN cd /rustless_host_engine && cargo build --release
WORKDIR /rustless_host_engine

CMD ["cargo", "run", "--release"]
EXPOSE 8080/tcp