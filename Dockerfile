# ─────────────────────────────────────────────────────────────────────────────
# Runtime-only image.
# The binary is compiled by the GitLab CI pipeline (cargo build --release)
# and copied in from the job artifact — no Rust toolchain needed here.
# ─────────────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for the service.
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Binary produced by the CI `build` job (artifact path: target/release/liljekvist-cc-mainpage)
COPY target/release/liljekvist-cc-mainpage ./liljekvist-cc-mainpage

# Tera templates (resolved relative to the working directory at runtime)
COPY templates ./templates

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

CMD ["./liljekvist-cc-mainpage"]

