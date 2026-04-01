## Implement BitTorrent Backend via `librqbit` (Full Scope, Brief Seeding)

### Summary
Implement `src/backends/torrent.rs` as a real BitTorrent backend for `magnet` and `.torrent` sources using embedded `librqbit`, with full peer/tracker/DHT behavior from the library, explicit `Downloading -> Seeding -> Completed` task lifecycle, and brief seeding (`min(configured_limit, 300s)`).

### Implementation Changes
- **Dependencies and runtime wiring**
  - Add `librqbit = { version = "8.1.1", default-features = false, features = ["rust-tls"] }`.
  - Add a shared torrent runtime/session manager (single process-wide session) owned by backend layer, initialized once and reused across torrent tasks.
  - Extend backend context to include `session_dir`, `default_seeding_time_limit_secs`, and `default_seeding_ratio_limit` so torrent behavior can honor global settings.

- **Backend interface updates (public/internal contract)**
  - Extend `BackendEvent` to support seeding lifecycle and upload progress:
    - Add `SeedingStarted`.
    - Extend `Progress` to include `uploaded_bytes` and `upload_rate_bps` (with defaults for non-torrent backends).
  - Update scheduler execution handlers to:
    - Transition to `TaskState::Seeding` on `SeedingStarted`.
    - Persist/upload metrics from progress events.
    - Keep existing behavior for HTTP/metalink backends unchanged.

- **Torrent backend behavior (`src/backends/torrent.rs`)**
  - Source ingestion:
    - `magnet:?` -> `AddTorrent::from_url(...)`.
    - `.torrent` URL -> fetch bytes, then `AddTorrent::from_bytes(...)`.
    - Local `.torrent` path -> read bytes, then `AddTorrent::from_bytes(...)`.
  - Output mapping:
    - Always treat `destination_path` as torrent output root directory (`download_dir/destination_path`), create directory if needed.
  - Protocol options (typed subset from `protocol_options`):
    - `disable_dht: bool`
    - `disable_trackers: bool`
    - `trackers: string[]`
    - `only_files: number[]`
    - `only_files_regex: string`
    - Unknown keys ignored safely.
  - Progress loop:
    - Emit `Started` once metadata/total size available.
    - Poll torrent stats at fixed interval (for example 1s), emit `Progress` with download/upload bytes and rates.
  - Completion and brief seeding:
    - On 100% payload download, emit `SeedingStarted`.
    - Seed until first satisfied condition:
      - elapsed seeding time reaches `min(effective_seeding_time_limit, 300s)`, or
      - ratio limit reached (if available from runtime stats), or
      - cancellation requested.
    - Then stop torrent activity and emit `Completed`.
  - Cancellation:
    - On cancel token, pause/stop torrent cleanly and return `BackendError::Cancelled`.

- **Error handling and lifecycle integration**
  - Add a dedicated backend error variant for torrent engine failures (instead of generic `Unsupported` strings).
  - Keep scheduler pause/resume contract:
    - Pause cancels backend run.
    - Resume starts backend run again, relying on torrent data/session persistence for restart continuity.
  - Update task-file deletion logic to handle torrent directories (not only single-file removal), using recursive delete when task source kind is `torrent` or `magnet`.

- **Docs updates**
  - Update backend support matrix in docs to mark `magnet/torrent` as implemented.
  - Document typed torrent `protocol_options`, destination-path-as-root-dir rule, and brief seeding semantics.

### Test Plan
- **Unit tests**
  - Parse and validate typed torrent `protocol_options` (valid subset, unknown keys ignored, invalid types rejected).
  - Seeding duration resolution: `min(task/global limit, 300s)` with fallback precedence.
  - Torrent source resolver for magnet, torrent URL, and local torrent file path.
- **Scheduler/backend integration tests**
  - Event/state sequence: `Started -> Progress -> SeedingStarted -> Progress -> Completed`.
  - Cancellation during downloading and seeding returns `Cancelled` and leaves task in paused/removed flow correctly.
  - Upload/download metrics propagation into stored task progress.
  - Remove task for torrent output directory deletes directory tree when requested.
- **Manual/acceptance scenario**
  - Magnet task with `auto_start` reaches downloading, then seeding, then completed within brief seeding window.
  - Resume after pause continues from existing data (no full restart from 0 when data exists).

### Assumptions
- `librqbit` v8.1.1 is the selected embedded engine.
- Brief seeding policy is explicitly enforced by Lux as `min(configured limit, 300s)`.
- `destination_path` for torrent tasks is treated as an output root directory (including single-file torrents).
- `protocol_options` uses the typed subset above in v1; unsupported keys are non-fatal and ignored.
