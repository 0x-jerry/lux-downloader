# Chrome Extension for Lux: Intercept Links + Embedded Web UI

## Summary

Build a Manifest V3 Chrome extension under `extension/` that:
- Intercepts file-like download links (and context-menu link actions),
- Cancels native browser download/navigation for those matched links,
- Creates Lux tasks via `POST /tasks` using configured server URL + bearer token,
- Provides a projected web interface via popup + full extension page for add/list/control flows.

## Implementation Changes

- Extension structure and assets:
  - Add `extension/manifest.json` (MV3), `background.js` (service worker), `content.js`, `popup.html/js`, `options.html/js`, `app.html/js` (full-page UI), and shared `styles.css`.
  - Keep all logic framework-free (plain JS) for minimal dependency and easy packaging.
- Core behavior:
  - `content.js` captures document click events in capture phase, detects file-like links (`download` attribute, known file extensions, magnet/torrent/metalink patterns), prevents default, and sends candidate URL + page context to background.
  - `background.js` owns Lux API calls and context menu actions (`Send link to Lux`), builds task payload, and returns success/failure to content/popup/app pages.
  - Add host + API permissions required for interception and Lux communication, plus `storage`, `contextMenus`, and `cookies`.
- Lux request mapping:
  - Always send `Authorization: Bearer <token>` from options config.
  - `POST /tasks` payload uses:
    - `source.kind: "auto"`,
    - `source.value: <intercepted URL>`,
    - `destination_path: <derived filename>`,
    - `settings.headers`: include `Referer` and `User-Agent`,
    - `settings.cookies`: include same-domain cookies from `chrome.cookies`,
    - `auto_start: true`.
- Projected web interface:
  - Popup: quick add form + latest task summary + “Open Dashboard”.
  - Full page (`app.html`): task list with periodic refresh (or WS optional if added), plus Pause/Resume/Remove controls.
  - Options page: server URL + auth token, save to `chrome.storage.sync`, validate with `/health` + authenticated `/tasks` probe.
- Internal interfaces (new public-ish extension contracts):
  - Runtime message actions: `intercept_add_task`, `manual_add_task`, `task_action`, `get_config`, `save_config`.
  - Storage schema: `{ baseUrl, authToken, interceptEnabled, includeCookies, includeReferer }` with defaults.
  - No Rust backend API changes required.

## Test Plan

- Manual functional tests:
  - Intercepted click on `.zip/.iso/.torrent` link creates Lux task and blocks native navigation/download.
  - Non-download links continue normal navigation.
  - Context-menu on link creates task.
  - Popup manual URL add works.
  - Dashboard lists tasks and lifecycle buttons call correct endpoints.
  - Options validation fails on bad token and succeeds on valid token.
- Edge-case tests:
  - URL without clear filename falls back to generated safe name.
  - Auth-required link succeeds with forwarded cookies/referrer where available.
  - Lux unreachable or `401` shows clear error in popup/dashboard toast/status.
- Regression checks:
  - No duplicate task creation for a single click.
  - Extension still functions after browser restart (config persisted).

## Assumptions and Defaults

- Chrome MV3 target only.
- Interception scope is file-like links + context-menu (not all navigation).
- Intercept action cancels native browser download/navigation on matched links.
- v1 UI scope is add/list/control (pause/resume/remove).
- Default Lux server config is user-managed via options page (not hardcoded), with saved settings required before first successful submit.
