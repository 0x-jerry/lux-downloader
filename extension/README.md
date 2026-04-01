# Lux Chrome Extension (WXT + Vue)

## Development

```bash
cd extension
npm install
npm run dev
```

## Production build

```bash
cd extension
npm run build
```

Build output is generated at `extension/.output/chrome-mv3`.

## Load in Chrome

1. Open `chrome://extensions`.
2. Enable Developer mode.
3. Click **Load unpacked**.
4. Select `extension/.output/chrome-mv3`.

## Features

- Intercepts file-like download links and sends them to Lux.
- Adds a context menu action: **Send link to Lux**.
- Popup for quick manual URL submit.
- Dashboard page for task list plus Lux URL/token and interception settings.
- Options page now redirects users to Dashboard.
