# Build stage
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssh-client && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/git-retail /usr/local/bin/git-retail
# NOTE: StrictHostKeyChecking is disabled for local development convenience only.
# Do not use this image in production without replacing this with a proper known_hosts configuration.
RUN mkdir -p /root/.ssh && \
    printf 'Host git-retailer\n  User git\n  StrictHostKeyChecking no\n  UserKnownHostsFile /dev/null\n' \
        > /root/.ssh/config && \
    chmod 700 /root/.ssh && chmod 600 /root/.ssh/config
COPY docker-entrypoint-retail.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
EXPOSE 7411
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
