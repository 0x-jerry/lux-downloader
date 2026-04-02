# Lux API

## Base URL

`http://<host>:<port>`

Default bind address is `0.0.0.0:8080`.

## Authentication

All endpoints except `/health` and `/metrics` require:

`Authorization: Bearer <token>`

Default token is `change-me` unless overridden by `AUTH_TOKEN` at startup.

## Content Types

- Request body: `application/json` (for REST endpoints with payload)
- Response body: `application/json` (except `/metrics` and WebSocket `/events`)

## Data Models

### SourceKind

- `auto`
- `url`
- `magnet`
- `torrent`
- `metalink`

### TaskState

- `queued`
- `metadata_fetching`
- `downloading`
- `seeding`
- `paused`
- `completed`
- `failed`
- `removed`

### TaskSpec

```json
{
  "source": {
    "kind": "auto",
    "value": "https://example.com/file.iso"
  },
  "destination_path": "file.iso",
  "overwrite_existing": false,
  "concurrency": 4,
  "checksum": "sha256:...",
  "protocol_options": {},
  "settings": {
    "priority": 0,
    "max_connections": 16,
    "max_download_rate_bps": 0,
    "max_upload_rate_bps": 0,
    "proxy": "http://127.0.0.1:7890",
    "headers": [{ "name": "User-Agent", "value": "lux/1.0" }],
    "cookies": [{ "name": "session", "value": "abc" }],
    "seeding_ratio_limit": 2.0,
    "seeding_time_limit_secs": 86400
  }
  ,
  "auto_start": true
}
```

Notes:

- `destination_path` must be relative to global `download_dir`.
- For `magnet` / `torrent`, `destination_path` is treated as the output root directory.
- Absolute paths and parent traversal (for example `../file.iso`) are rejected.
- `overwrite_existing` defaults to `false`; when `true`, existing destination files are ignored and download restarts from byte `0`.
- `source.kind` defaults to `auto` when omitted.
- With `auto`, the service detects kind from `source.value`:
  - `magnet:?` => `magnet`
  - URL ending with `.torrent` => `torrent`
  - URL ending with `.metalink`/`.meta4` or inline XML containing `<metalink` => `metalink`
  - `http|https|ftp|ftps|sftp` URLs => `url`
- `auto_start` defaults to `false`; when `true`, the service immediately resumes the created task.
- `protocol_options`, `checksum`, `cookies`, and some runtime settings are accepted but not yet enforced by all backends.
- For `magnet` / `torrent` tasks, `protocol_options` currently supports:
  - `disable_dht: bool`
  - `disable_trackers: bool`
  - `trackers: string[]`
  - `only_files: number[]`
  - `only_files_regex: string`
  - Unknown keys are ignored.
- If `settings` is omitted, it is stored as an empty settings object.

### TaskView

```json
{
  "id": "2c4ac748-cc57-4ccd-99cf-79039a75ebf8",
  "state": "queued",
  "spec": { "...": "TaskSpec" },
  "progress": {
    "downloaded_bytes": 0,
    "uploaded_bytes": 0,
    "total_bytes": null,
    "download_rate_bps": 0,
    "upload_rate_bps": 0,
    "verified": false
  },
  "error": null,
  "created_at": "2026-04-01T03:21:00.123Z",
  "updated_at": "2026-04-01T03:21:00.123Z"
}
```

### GlobalSettings

```json
{
  "auth_token": "change-me",
  "download_dir": "data/downloads",
  "session_dir": "data/session",
  "db_path": "data/lux.db",
  "global_connection_limit": 2048,
  "default_task_connection_limit": 16,
  "http_chunk_size_bytes": 4194304,
  "default_seeding_ratio_limit": 2.0,
  "default_seeding_time_limit_secs": 86400,
  "default_proxy": null
}
```

## Endpoints

### Health

#### `GET /health`

Returns service liveness.

Response `200`:

```json
{ "ok": true }
```

### Metrics

#### `GET /metrics`

Prometheus text metrics.

Response `200` (`text/plain; version=0.0.4`):

- `lux_tasks_queued`
- `lux_tasks_downloading`
- `lux_tasks_completed`
- `lux_active_download_rate_bps`
- `lux_active_upload_rate_bps`

### Create Task

#### `POST /tasks`

Creates a task.

Request body: `TaskSpec` plus optional:

```json
{
  "auto_start": true
}
```

Response `201`: `TaskView`

Important behavior:

- New tasks are created in `queued` state.
- If `auto_start=true`, the server attempts to start the task immediately.
- To start transfer, call `POST /tasks/{id}/resume`.

### List Tasks

#### `GET /tasks`

Query parameters:

- `state` (optional): one of TaskState strings
- `source_kind` (optional): `url|magnet|torrent|metalink`

Response `200`:

```json
{
  "items": [
    { "...": "TaskView" }
  ]
}
```

### Get Task

#### `GET /tasks/{id}`

Response `200`: `TaskView`

### Patch Task

#### `PATCH /tasks/{id}`

Patchable fields:

```json
{
  "settings": { "...": "TaskRuntimeSettings" },
  "concurrency": 8
}
```

Response `200`: updated `TaskView`

Notes:

- `concurrency` is clamped to at least `1` when provided.

### Pause Task

#### `POST /tasks/{id}/pause`

Response `200`: updated `TaskView`

### Resume Task

#### `POST /tasks/{id}/resume`

Transitions task to downloading and starts backend execution.

Response `200`: updated `TaskView`

### Restart Failed Task

#### `POST /tasks/{id}/restart`

Restarts a task that is currently in `failed` state by moving it back to `queued` and then starting execution again.

Response `200`: updated `TaskView`

### Remove Task

#### `POST /tasks/{id}/remove`

Query parameters:

- `delete_file` (optional, default `false`): when `true`, also attempts to delete the downloaded file from disk.

Response `200`: removed `TaskView` (state `removed`)

Behavior:

- The task is deleted from the database and no longer appears in `GET /tasks`.
- By default only task metadata is removed; file data is preserved unless `delete_file=true`.

### Verify Task

#### `POST /tasks/{id}/verify`

Transitions state to `metadata_fetching`.

Response `200`: updated `TaskView`

### Get Settings

#### `GET /settings`

Response `200`: `GlobalSettings`

### Update Settings

#### `PATCH /settings`

Request body: full `GlobalSettings` object.

Response `200`: updated `GlobalSettings`

Notes:

- Despite `PATCH`, this behaves like full replacement of global settings.

### Stats

#### `GET /stats`

Response `200`:

```json
{
  "task_counts": {
    "queued": 0,
    "metadata_fetching": 0,
    "downloading": 1,
    "seeding": 0,
    "paused": 0,
    "completed": 3,
    "failed": 0,
    "removed": 0
  },
  "active_download_rate_bps": 1048576,
  "active_upload_rate_bps": 0,
  "queued_tasks": 0,
  "active_tasks": 1
}
```

### Torrent Stats

#### `GET /tasks/{id}/torrent-stats`

Returns torrent runtime stats for a torrent/magnet task. Response includes both torrent `files`
and currently `connected_peers`.

Response `200` example:

```json
{
  "torrent_id": 3,
  "state": "live",
  "stats": {
    "state": "live",
    "file_progress": [1048576, 0],
    "error": null,
    "progress_bytes": 1048576,
    "uploaded_bytes": 0,
    "total_bytes": 8388608,
    "finished": false,
    "live": { "...": "librqbit live stats" }
  },
  "name": "ubuntu.iso",
  "info_hash": "0123456789abcdef0123456789abcdef01234567",
  "output_folder": "/downloads/ubuntu",
  "files": [
    {
      "name": "ubuntu.iso",
      "components": ["ubuntu.iso"],
      "length": 8388608,
      "included": true,
      "attributes": {}
    }
  ],
  "connected_peers": [
    {
      "address": "203.0.113.10:51413",
      "state": "live",
      "counters": {
        "incoming_connections": 0,
        "fetched_bytes": 524288,
        "total_time_connecting_ms": 18,
        "connection_attempts": 1,
        "connections": 1,
        "errors": 0,
        "fetched_chunks": 32,
        "downloaded_and_checked_pieces": 2,
        "total_piece_download_ms": 930,
        "times_stolen_from_me": 0,
        "times_i_stole": 0
      }
    }
  ],
  "connected_peer_count": 1
}
```

### Events WebSocket

#### `GET /events`

WebSocket endpoint with bearer auth.

Server sends JSON `EventEnvelope` messages:

```json
{
  "sequence": 12,
  "task_id": "2c4ac748-cc57-4ccd-99cf-79039a75ebf8",
  "event_type": "task_progress",
  "timestamp": "2026-04-01T03:22:11.004Z",
  "payload": { "...": "event-specific" }
}
```

Observed `event_type` values:

- `task_created`
- `task_settings_updated`
- `task_state_changed`
- `task_started`
- `task_progress`
- `task_completed`
- `task_failed`
- `task_warning`
- `settings_updated`

## Error Responses

General error shape:

```json
{ "error": "message" }
```

Status codes:

- `400 Bad Request`
  - unsupported source for available backends
  - invalid state transition
  - task already running
- `401 Unauthorized`
  - missing/invalid bearer token
- `404 Not Found`
  - task does not exist
- `500 Internal Server Error`
  - persistence/internal failures

## Backend Support Matrix (Current)

- `url` with `http`/`https`: implemented
- `metalink`: implemented (resolves first mirror URL, then downloads via HTTP backend)
- `url` with `ftp`/`ftps`: accepted but backend returns unsupported at runtime
- `url` with `sftp`: accepted but backend returns unsupported at runtime
- `magnet` / `torrent`: implemented (librqbit-backed, with brief seeding before completion)

## Quick cURL Examples

Create a task:

```bash
curl -X POST http://127.0.0.1:8080/tasks \
  -H 'Authorization: Bearer change-me' \
  -H 'Content-Type: application/json' \
  -d '{
    "source": { "value": "https://speed.hetzner.de/100MB.bin" },
    "destination_path": "100MB.bin",
    "concurrency": 4,
    "auto_start": true
  }'
```

Start task execution:

```bash
curl -X POST http://127.0.0.1:8080/tasks/<task-id>/resume \
  -H 'Authorization: Bearer change-me'
```

List downloading tasks:

```bash
curl 'http://127.0.0.1:8080/tasks?state=downloading' \
  -H 'Authorization: Bearer change-me'
```
