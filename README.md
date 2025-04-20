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
- **🧪 Built-in API Testing**: Test your endpoints directly from the dashboard
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

# Run with OpenAPI specification from current directory
docker run -p 8090:8090 \
  -v $(pwd)/openapi.json:/app/openapi.json \
  -e OPENAPI_FILE=/app/openapi.json \
  ghcr.io/arthurkowalsky/rust-mock:latest

# Run with OpenAPI specification from specific directory (change /path/to as needed)
docker run -p 8090:8090 \
  -v /path/to/openapi.json:/app/openapi.json \
  -e OPENAPI_FILE=/app/openapi.json \
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
```

Then run:

```bash
docker-compose up -d
```

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

RustMock comes pre-configured with sensible defaults (host: 0.0.0.0, port: 8090), but you can customize these settings when running the binary:

```bash
./RustMock --host 127.0.0.1 --port 3000
```

### Environment Variables

- `OPENAPI_FILE`: Path to your OpenAPI specification file (optional)

## 📡 API Reference

RustMock provides several admin endpoints to configure the mock server:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/__mock/config` | GET | Get current configuration |
| `/__mock/endpoints` | POST | Add a new endpoint |
| `/__mock/endpoints` | DELETE | Remove an endpoint |
| `/__mock/logs` | GET | Get request logs |
| `/__mock/logs` | DELETE | Clear logs |

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