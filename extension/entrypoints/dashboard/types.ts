import type { TorrentStats } from '../../shared'

export type { SourceKind, Task, TorrentFile, TorrentStats, ConnectedPeer } from '../../shared'

export interface TorrentDetailEntry {
  loading: boolean
  error: string
  data: TorrentStats | null
}
