export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue }

export interface LuxConfig {
  baseUrl: string
  authToken: string
  interceptEnabled: boolean
  includeCookies: boolean
  includeReferer: boolean
}

export interface LinkContext {
  url: string
  referer?: string
  destinationPath?: string
  overwrite?: boolean
}

export interface RuntimeResponse<T = unknown> {
  ok: boolean
  data?: T
  error?: string
}

export interface TaskActionRequest {
  id: string
  action: 'pause' | 'resume' | 'restart' | 'remove'
  deleteFile?: boolean
}

export interface TaskPatchRequest {
  source?: {
    kind?: 'auto' | 'url' | 'magnet' | 'torrent' | 'metalink'
    value: string
  }
  concurrency?: number
  settings?: Record<string, JsonValue>
}

export type SourceKind = 'auto' | 'url' | 'magnet' | 'torrent' | 'metalink'

export interface Task {
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

export interface ConnectedPeer {
  address?: string
  state?: string
  counters?: Record<string, number>
}

export interface TorrentFile {
  name: string
  length: number
  included?: boolean
}

export interface TorrentStats {
  torrent_id: number
  state: string
  name?: string
  info_hash?: string
  output_folder?: string
  files?: TorrentFile[]
  connected_peers?: ConnectedPeer[]
  connected_peer_count?: number
  stats?: {
    progress_bytes?: number
    uploaded_bytes?: number
    total_bytes?: number
    finished?: boolean
  }
}

export interface ListTasksResponse {
  items: Task[]
}
