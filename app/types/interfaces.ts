export type DownloadOs = 'windows' | 'macos' | 'linux'

export interface ReleasePlatform {
  signature?: string
  url?: string
}

export interface ReleaseJson {
  version?: string
  notes?: string
  pub_date?: string
  platforms?: Record<string, ReleasePlatform>
}

export interface DeviceProfile {
  architecture: string
  bitness: string
  platformVersion: string
}

export interface OsDownloadCard {
  os: DownloadOs
  label: string
  icon: string
  description: string
}

export interface ReleaseDownload {
  target: string
  url: string
  fileName: string
}

export interface GitHubReleaseAsset {
  name: string
  browser_download_url: string
}

export interface GitHubRelease {
  tag_name?: string
  body?: string
  published_at?: string
  assets?: GitHubReleaseAsset[]
}
