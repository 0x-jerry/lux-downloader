export function filenameFromUrl(rawUrl: string): string {
  try {
    const url = new URL(rawUrl)
    const parts = url.pathname.split('/').filter(Boolean)
    const candidate = decodeURIComponent(parts[parts.length - 1] || '').trim()
    if (candidate) {
      return sanitizeFilename(candidate)
    }
  } catch {
    // ignore parse errors
  }

  return `download-${Date.now()}`
}

function sanitizeFilename(name: string): string {
  return (
    name
      .replace(/[\\/:*?"<>|]/g, '_')
      .replace(/\s+/g, ' ')
      .trim()
      .slice(0, 180) || `download-${Date.now()}`
  )
}

export function isLikelyDownloadLink(rawHref: string, hasDownloadAttribute: boolean): boolean {
  if (hasDownloadAttribute) {
    return true
  }

  const href = rawHref.toLowerCase()

  if (href.startsWith('magnet:?')) {
    return true
  }

  const knownSuffixes = [
    '.zip',
    '.7z',
    '.rar',
    '.tar',
    '.gz',
    '.bz2',
    '.xz',
    '.iso',
    '.dmg',
    '.pkg',
    '.exe',
    '.msi',
    '.deb',
    '.rpm',
    '.apk',
    '.mp4',
    '.mkv',
    '.avi',
    '.mp3',
    '.flac',
    '.wav',
    '.pdf',
    '.epub',
    '.torrent',
    '.metalink',
    '.meta4',
  ]

  const stripped = href.split('#')[0].split('?')[0]
  return knownSuffixes.some((suffix) => stripped.endsWith(suffix))
}
