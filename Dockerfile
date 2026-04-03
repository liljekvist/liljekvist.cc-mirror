FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser

WORKDIR /app

COPY target/release/liljekvist-cc-mainpage ./liljekvist-cc-mainpage

COPY templates ./templates

COPY ascii_art ./ascii_art

COPY assets ./assets

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

CMD ["./liljekvist-cc-mainpage"]

