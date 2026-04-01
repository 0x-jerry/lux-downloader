# Rust Lux Service Design

## Summary

Build a container-oriented Rust download daemon that combines a multi-protocol transfer engine with a control-plane web service. v1 will support
HTTP/HTTPS, FTP, FTPS, SFTP, BitTorrent, magnet links, and Metalink, plus per-task HTTP proxy, gzip handling, custom headers, and cookie
management. The external interface will be REST for control and WebSocket for live events, with token-based auth and persistent task state across
restarts.

## Key Design

### Service shape

- Run as a single deployable daemon with three internal subsystems: control-api, scheduler, and protocol workers.
- Keep one process for v1 to reduce coordination complexity; isolate protocol-specific logic behind trait-based adapters so later split-out is
  possible.
- Package for containers with persistent volumes for downloads, session data, and DB files.

### Core runtime

- Use tokio as the async runtime.
- Use a central scheduler that owns task lifecycle, concurrency limits, retry policy, bandwidth limits, and resume orchestration.
- Represent every download as a Task with a normalized state machine: queued, metadata_fetching, downloading, seeding, paused, completed, failed,
  removed.
- Split downloads into protocol-native work units:
  - HTTP/FTP/SFTP: ranged segments or stream chunks.
  - Torrent: pieces and blocks.
  - Metalink: a metadata task that expands into one or more mirror-backed file tasks.
- Persist task metadata, progress snapshots, settings, and resumable checkpoints in SQLite for v1.
- Store partial payloads on disk with deterministic temp naming and atomic finalize on completion.

### Protocol support

- HTTP/HTTPS:
  - Use reqwest/hyper stack with range requests, redirects, gzip decompression, proxy support, custom headers, and cookie jar support.
  - Support per-task and global HTTP options; task-level settings override global defaults.
  - Resume only when server validators permit it; otherwise restart safely.
- FTP/FTPS:
  - Support passive mode by default, authenticated sessions, resume where server supports REST, and FTPS explicit TLS first.
  - Treat FTP and FTPS as the same adapter with transport-mode configuration.
- SFTP:
  - Use SSH-based file transfer with key or password auth and resumable reads based on remote file size.
- BitTorrent:
  - Support .torrent and magnet ingestion, DHT, PEX, TCP/UDP trackers, per-torrent seeding policy, piece verification, and peer session
    management.
  - Persist torrent session state, bitfields, and tracker/DHT metadata needed for restart recovery.
  - Expose seeding limits by ratio and time, with defaults chosen in config.
- Metalink:
  - Parse Metalink XML, verify hashes, expand mirrors, and feed the scheduler with mirror candidates and checksum policy.
  - Prefer mirrors by health and observed throughput; fall back automatically.

### Web service

- REST API:
  - POST /tasks create download from URL, magnet, torrent file reference, or metalink payload/reference.
  - GET /tasks list tasks with filters.
  - GET /tasks/{id} fetch task detail.
  - POST /tasks/{id}/pause|resume|remove|verify lifecycle operations.
  - PATCH /tasks/{id} mutate runtime settings such as speed caps, headers, cookies, proxy, seeding policy.
  - GET /settings and PATCH /settings for daemon defaults.
  - GET /stats for global bandwidth, queue, peer, and storage metrics.
- WebSocket stream:
  - GET /events pushes task progress, lifecycle changes, peer/tracker updates, and warnings/errors.
- Auth:
  - Require bearer token on all non-health endpoints.
  - Provide a static token bootstrap via env/config for v1; no built-in user system.
- API contract:
  - All tasks use a unified schema with protocol-specific sub-objects instead of separate top-level task types.
  - Event payloads include monotonic sequence numbers so clients can recover from disconnects.

### Important public types/interfaces

- TaskSpec: input object for creating a task; includes source kind, destination path, concurrency, proxy, headers, cookies, checksum policy, and
  protocol-specific options.
- TaskView: canonical API response object with normalized state, progress, rates, errors, and protocol detail.
- GlobalSettings: daemon-wide limits, directories, token, network defaults, seeding defaults, and persistence settings.
- EventEnvelope: typed WebSocket event wrapper with sequence, task_id, event_type, timestamp, and payload.

## Implementation Notes

- Prefer axum for the web layer, sqlx with SQLite for persistence, and serde for API contracts.
- Build protocol workers behind a common TransferBackend interface so the scheduler can remain protocol-agnostic.
- Use a disk-backed piece/segment manager shared by HTTP-family and torrent code to avoid duplicate resume logic.
- Separate cookie handling into:
  - Global cookie jar for reusable sessions.
  - Optional task-local cookies that do not leak across tasks.
- Support proxy configuration at global and task scopes; for v1, only HTTP/HTTPS proxying is guaranteed for HTTP-family traffic, with SOCKS
  deferred unless library support is selected later.
- Add structured logging and metrics from day one; export Prometheus-compatible metrics endpoint for container deployments.
- Define backpressure and fairness rules in the scheduler:
  - Global connection cap.
  - Per-task concurrency cap.
  - Bandwidth throttling.
  - Priority queue for queued tasks.

## Test Plan

- Unit tests for task state transitions, resume checkpoint logic, header/cookie/proxy precedence, checksum validation, and Metalink expansion.
- Integration tests for:
  - HTTP range/resume and gzip.
  - Authenticated FTP/FTPS and SFTP downloads.
  - Proxy routing for HTTP tasks.
  - Cookie persistence and custom header injection.
  - REST lifecycle operations and WebSocket event ordering.
- Torrent integration tests for magnet bootstrap, tracker communication, piece verification, pause/resume, and seeding stop conditions.
- Restart-recovery tests to confirm persistent tasks resume correctly after daemon restart.
- Failure-path tests for checksum mismatch, credential failure, unreachable mirrors, tracker failure, partial file corruption, and disk-full
  behavior.

## Assumptions And Defaults

- v1 is API-first; no bundled frontend UI is included, but the API and WebSocket surface are designed to back a future frontend.
- “(S)FTP” is implemented as FTP, FTPS, and SFTP.
- SQLite is the default persistent store for v1; external DB support is out of scope initially.
- Bearer token auth is the only built-in auth model in v1.
- REST + WebSocket is preferred over aria2-compatible JSON-RPC, so migration compatibility is not a v1 goal.
- Full torrent support includes DHT, PEX, and UDP trackers; advanced private-tracker edge cases and exotic extensions can be deferred if they
  threaten delivery.
