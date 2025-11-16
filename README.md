# RustMock

<p align="center">
  <img src="images/logo.png" alt="RustMock Logo" width="250">
</p>

<p align="center">
  <a href="https://github.com/arthurkowalsky/Rust-Mock/actions"><img src="https://img.shields.io/github/actions/workflow/status/arthurkowalsky/Rust-Mock/ci.yml" alt="Build Status"></a>
  <a href="https://github.com/arthurkowalsky/Rust-Mock/releases"><img src="https://img.shields.io/github/v/release/arthurkowalsky/Rust-Mock" alt="Version"></a>
  <a href="https://github.com/arthurkowalsky/Rust-Mock/blob/main/LICENSE"><img src="https://img.shields.io/github/license/arthurkowalsky/Rust-Mock" alt="License"></a>
  <a href="https://github.com/arthurkowalsky/Rust-Mock/pkgs/container/rust-mock"><img src="https://img.shields.io/badge/ghcr.io-package-blue" alt="GitHub Package"></a>
  <a href="https://github.com/arthurkowalsky/Rust-Mock/pkgs/container/rust-mock"><img src="https://img.shields.io/github/repo-size/arthurkowalsky/Rust-Mock" alt="Image Size"></a>
</p>

> **⚡ Lightning-fast, lightweight API mock server built in Rust with a sleek React dashboard**

RustMock is an elegant, high-performance mock server designed for developers who need reliable, fast API mocks for development and testing. At just **~35MB** Docker image size, it's incredibly lightweight while providing powerful functionality.

## 🚀 Features

- **⚡ Blazing Fast Performance**: Built on Rust and Actix Web for exceptional speed
- **🎯 Dynamic API Mocking**: Create and configure mock endpoints on-the-fly
- **💻 Beautiful UI Dashboard**: Sleek React interface for managing all aspects of your mock server
- **🔍 Comprehensive Request Logging**: Monitor and inspect all incoming requests
- **📝 OpenAPI Support**: Automatically create mock endpoints from your OpenAPI spec
- **📥 OpenAPI Import/Export**: Import endpoints from OpenAPI specifications and export your configuration to OpenAPI 3.0 format
- **🧪 Built-in API Testing**: Test your endpoints directly from the dashboard
- **🔄 Smart Proxy Mode**: Hybrid mock + real API - selectively mock endpoints while proxying others to production
- **🐳 Docker Ready**: Get started in seconds with pre-built Docker images

## 📸 Screenshots

<table>
  <tr>
    <td><img src="images/endpoints_list.png" alt="Endpoints List" /></td>
    <td><img src="images/log_details.png" alt="Log Details" /></td>
  </tr>
  <tr>
    <td><img src="images/logs_list.png" alt="Logs List" /></td>
    <td><img src="images/test_endpoint.png" alt="Test Endpoint" /></td>
  </tr>
</table>

## 🔧 Quick Start

### Using Docker (Recommended)

```bash
# Basic run command
docker run -p 8090:8090 ghcr.io/arthurkowalsky/rust-mock:latest

# Run with OpenAPI specification
docker run -p 8090:8090 \
  -v $(pwd)/openapi.json:/app/openapi.json \
  -e OPENAPI_FILE=/app/openapi.json \
  ghcr.io/arthurkowalsky/rust-mock:latest

# Run with Smart Proxy Mode - proxy unmocked endpoints to production
docker run -p 8090:8090 \
  -e DEFAULT_PROXY_URL=https://api.production.com \
  ghcr.io/arthurkowalsky/rust-mock:latest

# Combined: OpenAPI + Smart Proxy Mode
docker run -p 8090:8090 \
  -v $(pwd)/openapi.json:/app/openapi.json \
  -e OPENAPI_FILE=/app/openapi.json \
  -e DEFAULT_PROXY_URL=https://api.production.com \
  ghcr.io/arthurkowalsky/rust-mock:latest
```

### Using Docker Compose

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
      - DEFAULT_PROXY_URL=https://api.production.com  # Optional: Enable Smart Proxy Mode
```

Then run:

```bash
docker-compose up -d
```

**Pro Tip**: Copy `.env.example` to `.env` and configure your settings there, then use `env_file` in docker-compose.

### Building from Source

```bash
git clone https://github.com/arthurkowalsky/Rust-Mock.git
cd Rust-Mock
cargo build --release
./target/release/RustMock
```

## 📖 Usage

### Accessing the Dashboard

Once running, access the dashboard at:
```
http://localhost:8090
```

### Configuration Options

RustMock supports configuration via CLI arguments and environment variables:

**CLI Arguments:**
```bash
./RustMock --host 127.0.0.1 --port 3000 --default-proxy-url https://api.example.com
```

**Available Options:**

| Argument | Default | Description |
|----------|---------|-------------|
| `--host` | `0.0.0.0` | Server host address |
| `--port` | `8090` | Server port |
| `--default-proxy-url` | _none_ | Default proxy URL for Smart Proxy Mode |

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `OPENAPI_FILE` | Path to OpenAPI specification file | `/app/openapi.json` |
| `DEFAULT_PROXY_URL` | Default proxy URL for unmocked endpoints (Smart Proxy Mode) | `https://api.production.com` |

**Note**: CLI arguments take precedence over environment variables. See `.env.example` for detailed configuration examples.

## 📡 API Reference

RustMock provides several admin endpoints to configure the mock server:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/__mock/config` | GET | Get current configuration |
| `/__mock/endpoints` | POST | Add a new endpoint |
| `/__mock/endpoints` | DELETE | Remove an endpoint |
| `/__mock/logs` | GET | Get request logs |
| `/__mock/logs` | DELETE | Clear logs |
| `/__mock/import` | POST | Import endpoints from OpenAPI specification |
| `/__mock/export` | GET | Export endpoints as OpenAPI 3.0 specification |
| `/__mock/proxy` | GET | Get default proxy configuration |
| `/__mock/proxy` | POST | Set default proxy URL |
| `/__mock/proxy` | DELETE | Remove default proxy |

### Adding a Mock Endpoint

```http
POST /__mock/endpoints
Content-Type: application/json

{
  "method": "POST",
  "path": "/api/users",
  "response": {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com"
  },
  "status": 201,
  "headers": {
    "Content-Type": "application/json",
    "X-Custom-Header": "custom-value"
  }
}
```

### Importing from OpenAPI Specification

Import endpoints from an OpenAPI 3.0 specification file:

```http
POST /__mock/import
Content-Type: application/json

{
  "openapi_spec": {
    "openapi": "3.0.0",
    "info": {
      "title": "My API",
      "version": "1.0.0"
    },
    "paths": {
      "/api/users": {
        "get": {
          "summary": "Get users",
          "responses": {
            "200": {
              "description": "Success",
              "content": {
                "application/json": {
                  "example": [{"id": 1, "name": "John"}]
                }
              }
            }
          }
        }
      }
    }
  }
}
```

**Response:**
```json
{
  "imported": true,
  "count": 1,
  "endpoints": [
    {
      "method": "GET",
      "path": "/api/users",
      "status": 200
    }
  ]
}
```

### Exporting to OpenAPI Specification

Export all configured endpoints as an OpenAPI 3.0 specification:

```http
GET /__mock/export
```

**Response:**
```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Mock API",
    "description": "Exported from Rust-Mock server",
    "version": "1.0.0"
  },
  "paths": {
    "/api/users": {
      "get": {
        "summary": "GET /api/users",
        "operationId": "get_api_users",
        "responses": {
          "200": {
            "description": "Successful response with status 200",
            "content": {
              "application/json": {
                "example": [{"id": 1, "name": "John"}],
                "schema": {
                  "type": "object"
                }
              }
            }
          }
        }
      }
    }
  }
}
```

### Using the Dashboard for Import/Export

The React dashboard provides intuitive buttons for importing and exporting OpenAPI specifications:

1. **Import OpenAPI**: Click the "Import OpenAPI" button and select your OpenAPI JSON file
2. **Export OpenAPI**: Click the "Export OpenAPI" button to download your endpoints as an OpenAPI 3.0 specification
3. **Export JSON**: Click the "Export JSON" button to download endpoints in the internal format

## 🔄 Smart Proxy Mode

Smart Proxy Mode enables **hybrid mock + real API testing**. Selectively mock specific endpoints while proxying all other requests to a real API.

### Quick Example

```bash
# Set default proxy to production
export DEFAULT_PROXY_URL="https://api.production.com"
./RustMock

# Mock only the new endpoint you're testing
curl -X POST http://localhost:8090/__mock/endpoints \
  -H "Content-Type: application/json" \
  -d '{
    "method": "POST",
    "path": "/api/v2/new-feature",
    "response": {"status": "success", "data": {"id": 123}},
    "status": 201
  }'

# Result:
# - POST /api/v2/new-feature → Returns mock response
# - GET /api/users → Proxied to production API
# - GET /api/orders → Proxied to production API
# - All other endpoints → Proxied to production API
```

### Use Cases

1. **Test New Endpoints with Production Data**
   - Mock your new endpoint while keeping real data for everything else
   - Ideal for integration testing without affecting production

2. **Simulate Errors on Specific Endpoints**
   - Mock error responses on selected endpoints
   - Test error handling without breaking your actual API

3. **Multi-Environment Testing**
   - Route different endpoints to different environments
   - Example: Auth → staging, Payments → production

4. **Debug with Real Data**
   - Use production data for stable endpoints
   - Mock only the problematic endpoint for debugging

### Configuration

**Runtime Configuration via API:**
```bash
# Set default proxy
curl -X POST http://localhost:8090/__mock/proxy \
  -H "Content-Type: application/json" \
  -d '{"url": "https://api.production.com"}'

# Get current proxy config
curl http://localhost:8090/__mock/proxy

# Remove default proxy
curl -X DELETE http://localhost:8090/__mock/proxy
```

**Per-Endpoint Proxy:**
```bash
# Route specific endpoint to different URL
curl -X POST http://localhost:8090/__mock/endpoints \
  -H "Content-Type: application/json" \
  -d '{
    "method": "GET",
    "path": "/api/users",
    "proxy_url": "https://api.staging.com",
    "response": {},
    "status": 200
  }'
```

**UI Configuration:**
- Navigate to **Settings** page in the dashboard
- Configure default proxy URL
- View current proxy status and source (ENV/CLI/Runtime)

For detailed documentation and advanced usage, see [PROXY_MODE.md](./PROXY_MODE.md)

## 🏗️ Architecture

RustMock consists of two main components:

1. **Backend Server**: Written in Rust using Actix Web, providing exceptional performance and reliability
2. **Frontend Dashboard**: Built with React and modern UI components for an intuitive experience

The server supports dynamic endpoint creation, request validation against OpenAPI schemas, and comprehensive request logging.

## 💡 Why RustMock?

- **Performance**: Built on Rust's blazing-fast performance
- **Resource Efficiency**: Only ~35MB Docker image and minimal memory footprint
- **Developer Experience**: Intuitive UI for managing all aspects of your mock server
- **Integration**: Works seamlessly with your existing development workflow
- **No Dependencies**: Self-contained binary with everything you need

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.