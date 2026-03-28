# ALICE WebSocket SaaS

WebSocket gateway management powered by ALICE. Manage connections, broadcast messages, rooms, and configuration via a simple REST API.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Status

| Check | Status |
|-------|--------|
| `cargo check` | passing |
| API health | `/health` |

## Quick Start

```bash
docker compose up -d
```

API Gateway: http://localhost:8131

## Architecture

```
Client
  |
  v
API Gateway          :8131
  |
  v
WebSocket Engine     :8132-internal
(connection registry, room management, broadcast)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/api/v1/ws/connections` | List active connections |
| `POST` | `/api/v1/ws/broadcast` | Broadcast message to connections |
| `POST` | `/api/v1/ws/rooms` | Create or join a room |
| `POST` | `/api/v1/ws/config` | Update gateway configuration |
| `GET`  | `/api/v1/ws/stats` | Retrieve gateway statistics |
| `GET`  | `/health` | Service health check |

### broadcast

```json
POST /api/v1/ws/broadcast
{
  "room": "notifications",
  "message": "Hello World",
  "type": "text"
}
```

### rooms

```json
POST /api/v1/ws/rooms
{
  "name": "chat-room-1",
  "max_connections": 100,
  "ttl_secs": 3600
}
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WS_ADDR` | `0.0.0.0:8131` | Core engine bind address |
| `GATEWAY_ADDR` | `0.0.0.0:8130` | API gateway bind address |

## License

AGPL-3.0. Commercial dual-license available — contact for pricing.
