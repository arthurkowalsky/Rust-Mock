FROM node:22-alpine AS ui-builder
WORKDIR /app/ui
COPY ui/package*.json ./
RUN npm install
COPY ui/ ./
RUN npm run build

FROM rust:1.91-slim AS rust-builder
WORKDIR /app
COPY Cargo.* ./
COPY src/ ./src/
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=rust-builder /app/target/release/RustMock /app/RustMock
COPY --from=ui-builder /app/ui/dist /app/ui/dist

EXPOSE 8090
USER nonroot
CMD ["/app/RustMock"]