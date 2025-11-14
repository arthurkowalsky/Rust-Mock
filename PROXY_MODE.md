# Smart Proxy Mode - Usage Guide

## Overview

Smart Proxy Mode enables **hybrid mock + real API testing**. You can:
- Keep most endpoints mocked while proxying specific ones to production
- Test new endpoints in isolation while using real data for the rest
- Simulate errors on specific endpoints without affecting others
- Mix and match mock and proxy behaviors for ultimate flexibility

## Quick Start

### 1. Global Default Proxy

Set a default proxy URL that catches all unmocked requests:

```bash
# Via environment variable
export DEFAULT_PROXY_URL="https://api.production.com"
./rust-mock

# Via CLI argument
./rust-mock --default-proxy-url https://api.production.com
```

### 2. Per-Endpoint Proxy

Add an endpoint with a `proxy_url` field:

```bash
curl -X POST http://localhost:8090/__mock/endpoints \
  -H "Content-Type: application/json" \
  -d '{
    "method": "GET",
    "path": "/api/users",
    "proxy_url": "https://api.production.com",
    "response": {},
    "status": 200
  }'
```

### 3. Runtime Proxy Configuration

Manage the global proxy at runtime:

```bash
# Set default proxy
curl -X POST http://localhost:8090/__mock/proxy \
  -H "Content-Type: application/json" \
  -d '{"url": "https://api.production.com"}'

# Get current proxy config
curl http://localhost:8090/__mock/proxy

# Delete default proxy
curl -X DELETE http://localhost:8090/__mock/proxy
```

## How It Works

### Request Flow

```
Incoming Request
    ↓
Check dynamic endpoints
    ↓
┌─────────────────────┐
│ Endpoint found?     │
└──────┬──────────────┘
       │
    ┌──┴──┐
   YES   NO
    │     │
    ↓     ↓
Has      Default
proxy?   proxy?
    │     │
  ┌─┴─┐ ┌─┴─┐
 YES NO YES NO
  │   │   │   │
  ↓   ↓   ↓   ↓
Proxy Mock Proxy 404
```

### Priority Order

1. **Endpoint with `proxy_url`** → Forward to specified URL
2. **Endpoint with `response`** → Return mock response
3. **Global `default_proxy_url`** → Forward to default URL
4. **No match** → 404 Not Found

## Use Cases

### Case 1: Test New Endpoint with Production Data

```bash
# Set global proxy to production
export DEFAULT_PROXY_URL="https://api.prod.com"

# Add ONLY the new endpoint as a mock
curl -X POST http://localhost:8090/__mock/endpoints \
  -d '{
    "method": "POST",
    "path": "/api/v2/new-feature",
    "response": {"status": "success", "data": {"id": 123}},
    "status": 201
  }'

# Result:
# - POST /api/v2/new-feature → MOCK (201)
# - GET /api/users → PROXY to prod
# - GET /api/orders → PROXY to prod
# - Everything else → PROXY to prod
```

### Case 2: Simulate Errors on Specific Endpoints

```bash
# Set global proxy
curl -X POST http://localhost:8090/__mock/proxy \
  -d '{"url": "https://api.prod.com"}'

# Mock a payment failure
curl -X POST http://localhost:8090/__mock/endpoints \
  -d '{
    "method": "POST",
    "path": "/api/payment",
    "response": {"error": "Payment gateway timeout"},
    "status": 503
  }'

# Result:
# - POST /api/payment → MOCK (503 error)
# - All other endpoints → PROXY to prod
```

### Case 3: Multi-Environment Proxying

```bash
# Auth goes to staging
curl -X POST http://localhost:8090/__mock/endpoints \
  -d '{
    "method": "POST",
    "path": "/api/auth/login",
    "proxy_url": "https://auth.staging.com",
    "response": {},
    "status": 200
  }'

# Payments go to production
curl -X POST http://localhost:8090/__mock/endpoints \
  -d '{
    "method": "POST",
    "path": "/api/payments",
    "proxy_url": "https://payments.prod.com",
    "response": {},
    "status": 200
  }'

# Result:
# - POST /api/auth/login → PROXY to staging
# - POST /api/payments → PROXY to production
# - Different services, different environments!
```

### Case 4: Test Edge Cases Without Touching Production

```bash
# Most requests go to prod
export DEFAULT_PROXY_URL="https://api.prod.com"

# But test rate limiting locally
curl -X POST http://localhost:8090/__mock/endpoints \
  -d '{
    "method": "GET",
    "path": "/api/users",
    "response": {"error": "Rate limit exceeded"},
    "status": 429,
    "headers": {
      "Retry-After": "60",
      "X-RateLimit-Remaining": "0"
    }
  }'
```

## Features

### ✅ Header Forwarding

All request headers (except `host`, `connection`, `transfer-encoding`) are forwarded to the upstream server.

### ✅ Query Parameter Forwarding

Query parameters are preserved and forwarded to the upstream:

```bash
GET /api/users?page=2&limit=10
# Proxied to: https://api.prod.com/api/users?page=2&limit=10
```

### ✅ Body Forwarding

Request bodies are forwarded as-is:

```bash
POST /api/users
Body: {"name": "John", "email": "john@example.com"}
# Proxied with the same body
```

### ✅ Response Passthrough

Proxy responses are returned unchanged:
- Status codes preserved
- Response headers forwarded
- Response body forwarded

### ✅ Request Logging

All proxied requests are logged with:
- Original request data
- Response data
- `proxied_to` field showing the target URL

Example log entry:

```json
{
  "method": "GET",
  "path": "/api/users",
  "status": 200,
  "response_body": {"users": [...]},
  "proxied_to": "https://api.prod.com/api/users",
  "timestamp": "2025-01-14T12:34:56Z"
}
```

### ✅ Error Handling

If a proxy request fails:
- Returns `502 Bad Gateway`
- Includes error details in response body
- Logs the failure

```json
{
  "error": "Proxy request failed",
  "details": "Connection timeout"
}
```

## Configuration

### Environment Variables

- `DEFAULT_PROXY_URL` - Default proxy URL for unmocked endpoints

### CLI Arguments

```bash
./rust-mock --help
# Shows:
#   --default-proxy-url <URL>    Default proxy URL for unmocked endpoints
```

### Endpoint Configuration

```json
{
  "method": "GET",
  "path": "/api/users",
  "response": {"users": []},      // Used if proxy_url is not set
  "status": 200,                  // Used if proxy_url is not set
  "headers": {...},               // Custom headers (for mock mode)
  "proxy_url": "https://api.prod.com"  // Optional: proxy to this URL
}
```

## Testing

Run the proxy mode tests:

```bash
cargo test --test integration_tests -- proxy
```

Tests include:
- Proxy configuration endpoints
- Per-endpoint proxying
- Default proxy fallback
- Mixed mock + proxy mode
- Query parameter forwarding
- POST body forwarding
- Proxy failure handling

## UI Integration

The web UI includes:
- **Proxy tab** in endpoint form for setting `proxy_url`
- **Proxy configuration panel** for global proxy settings
- **Proxy indicator** in logs showing where requests were proxied

## Performance

- Timeout: 30 seconds per proxy request
- Connection pooling: Handled by `reqwest`
- Concurrent requests: Supported
- No caching: All requests forwarded in real-time

## Security

⚠️ **Important Security Notes:**

- Proxy mode forwards ALL headers and data to upstream
- Ensure upstream URLs are trusted
- Use HTTPS for upstream URLs in production
- Sensitive headers like `Authorization` are forwarded

## Troubleshooting

### Proxy returns 502

Check:
1. Is the upstream URL accessible?
2. Does it return valid HTTP responses?
3. Check logs for detailed error message

### Proxy ignores custom headers

Custom `headers` field only applies to mock mode. In proxy mode, upstream response headers are used.

### Query params not forwarded

Check URL encoding. Query params are forwarded automatically.

## Examples

See `tests/integration_tests.rs` for comprehensive examples.

## API Reference

### GET `/__mock/proxy`

Get current default proxy configuration.

**Response:**
```json
{
  "proxy_url": "https://api.prod.com",
  "enabled": true
}
```

### POST `/__mock/proxy`

Set default proxy URL.

**Request:**
```json
{
  "url": "https://api.prod.com"
}
```

**Response:**
```json
{
  "proxy_url": "https://api.prod.com",
  "enabled": true
}
```

### DELETE `/__mock/proxy`

Remove default proxy.

**Response:**
```json
{
  "deleted": true
}
```

---

Happy proxying! 🚀
