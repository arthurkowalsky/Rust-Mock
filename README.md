# RustMock

<p align="center">
  <img src="images/logo.png" alt="RustMock Logo" width="200">
</p>

<p align="center">
  <a href="https://crates.io/crates/RustMock"><img src="https://img.shields.io/crates/v/RustMock" alt="Crates.io"></a>
  <a href="https://crates.io/crates/RustMock"><img src="https://img.shields.io/crates/d/RustMock" alt="Downloads"></a>
  <a href="https://github.com/arthurkowalsky/Rust-Mock/actions"><img src="https://img.shields.io/github/actions/workflow/status/arthurkowalsky/Rust-Mock/ci.yml" alt="Build Status"></a>
  <a href="https://github.com/arthurkowalsky/Rust-Mock/blob/main/LICENSE"><img src="https://img.shields.io/github/license/arthurkowalsky/Rust-Mock" alt="License"></a>
</p>

> **Lightning-fast API mock server** built in Rust with beautiful React dashboard

~35MB Docker image | Blazing fast | Full OpenAPI support | Smart proxy mode

---

## Quick Start

**Docker:**
```bash
docker run -p 8090:8090 ghcr.io/arthurkowalsky/rust-mock:latest
```

**CLI (Cargo):**
```bash
cargo install RustMock
mokku
```

Open **http://localhost:8090** - done!

---

## Features

- **Blazing Fast** - Built on Rust + Actix Web, ~35MB Docker image
- **CLI & Docker** - `mokku` command or Docker one-liner
- **OpenAPI Support** - Import/export specs, auto-generate endpoints
- **React Dashboard** - Beautiful UI for managing mocks and viewing logs
- **Smart Proxy** - Mix mocked endpoints with real API calls
- **Request Logging** - Monitor and inspect all incoming requests

---

## CLI Reference

### Installation

```bash
cargo install RustMock
```

### Commands

| Command | Description |
|---------|-------------|
| `mokku` | Interactive mode - guided setup |
| `mokku server` | Start server directly |
| `mokku import <file>` | Import OpenAPI spec |
| `mokku mock <method> <path> [status] [body]` | Quick mock creation |

### Examples

```bash
# Start with custom port
mokku server -p 3000

# Start with proxy to production API
mokku --proxy https://api.production.com

# Import OpenAPI and start server
mokku import openapi.yaml --start --open

# Quick mock endpoint
mokku mock POST /api/users 201 '{"id": 1, "name": "John"}'
```

### Global Flags

| Flag | Description |
|------|-------------|
| `-p, --port <PORT>` | Server port (default: 8090) |
| `--host <HOST>` | Bind address (default: 0.0.0.0) |
| `--proxy <URL>` | Default proxy URL |
| `-o, --open` | Auto-open browser |

---

## Docker

### Basic Usage

```bash
docker run -p 8090:8090 ghcr.io/arthurkowalsky/rust-mock:latest
```

### With OpenAPI Import

```bash
docker run -p 8090:8090 \
  -v $(pwd)/openapi.json:/app/openapi.json \
  -e OPENAPI_FILE=/app/openapi.json \
  ghcr.io/arthurkowalsky/rust-mock:latest
```

### Docker Compose

```yaml
version: '3'
services:
  rustmock:
    image: ghcr.io/arthurkowalsky/rust-mock:latest
    ports:
      - "8090:8090"
    volumes:
      - ./openapi.json:/app/openapi.json
    environment:
      - OPENAPI_FILE=/app/openapi.json
      - DEFAULT_PROXY_URL=https://api.production.com  # optional
```

---

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/__mock/endpoints` | POST | Add endpoint |
| `/__mock/endpoints` | PUT | Update endpoint |
| `/__mock/endpoints` | DELETE | Remove endpoint |
| `/__mock/config` | GET | Get all endpoints |
| `/__mock/logs` | GET | Get request logs |
| `/__mock/logs` | DELETE | Clear logs |
| `/__mock/import` | POST | Import OpenAPI spec |
| `/__mock/export` | GET | Export as OpenAPI |
| `/__mock/proxy` | GET/POST/DELETE | Manage proxy |

### Add Endpoint

```bash
curl -X POST http://localhost:8090/__mock/endpoints \
  -H "Content-Type: application/json" \
  -d '{
    "method": "GET",
    "path": "/api/users",
    "response": [{"id": 1, "name": "John"}],
    "status": 200
  }'
```

### Import OpenAPI

```bash
curl -X POST http://localhost:8090/__mock/import \
  -H "Content-Type: application/json" \
  -d '{"openapi_spec": <your-openapi-json>}'
```

---

## Proxy Mode

Route unmocked requests to a real API:

```bash
# Via CLI
mokku --proxy https://api.production.com

# Via Docker
docker run -p 8090:8090 -e DEFAULT_PROXY_URL=https://api.production.com ghcr.io/arthurkowalsky/rust-mock:latest

# Via API
curl -X POST http://localhost:8090/__mock/proxy \
  -H "Content-Type: application/json" \
  -d '{"url": "https://api.production.com"}'
```

Mock specific endpoints while proxying the rest to production.

[Full proxy documentation →](./PROXY_MODE.md)

---

## Configuration

### CLI Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `--host` | `0.0.0.0` | Server host |
| `--port` | `8090` | Server port |
| `--default-proxy-url` | - | Proxy URL for unmocked requests |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAPI_FILE` | Path to OpenAPI spec for auto-import |
| `DEFAULT_PROXY_URL` | Default proxy URL |

CLI arguments take precedence over environment variables.

---

## Architecture

RustMock consists of:

- **Backend**: Rust + Actix Web for exceptional performance
- **Frontend**: React dashboard for intuitive management
- **Features**: Dynamic endpoints, OpenAPI validation, comprehensive logging

## Why RustMock?

- **Performance** - Built on Rust's blazing-fast foundation
- **Lightweight** - ~35MB Docker image, minimal memory footprint
- **Developer UX** - Beautiful dashboard, intuitive CLI
- **Zero config** - Works out of the box

---

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Open Pull Request

## License

MIT License - see [LICENSE](LICENSE)
