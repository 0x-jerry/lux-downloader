export type SourceKind = 'auto' | 'url' | 'magnet' | 'torrent' | 'metalink'

export type Task = {
  id: string
  state: string
  progress: {
    downloaded_bytes: number
    uploaded_bytes: number
    total_bytes: number | null
    download_rate_bps: number
    upload_rate_bps: number
    verified: boolean
  }
  spec: {
    source: {
      kind: SourceKind
      value: string
    }
    destination_path: string
  }
}

export type TorrentFile = {
  name: string
  length: number
  included?: boolean
}

export type TorrentStats = {
  torrent_id: number
  state: string
  name?: string
  info_hash?: string
  output_folder?: string
  files?: TorrentFile[]
  connected_peers?: Array<{
    address?: string
    state?: string
    counters?: Record<string, unknown>
    [key: string]: unknown
  }>
  connected_peer_count?: number
  stats?: {
    progress_bytes?: number
    uploaded_bytes?: number
    total_bytes?: number
    finished?: boolean
  }
}

export type TorrentDetailEntry = {
  loading: boolean
  error: string
  data: TorrentStats | null
}
