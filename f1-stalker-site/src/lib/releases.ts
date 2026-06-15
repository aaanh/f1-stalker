export const APP_VERSION = "0.1.1" as const

export type ReleaseDownload = {
  label: string
  href: string
  fileName: string
}

export const releaseDownloads: ReleaseDownload[] = [
  {
    label: "macOS Universal",
    href: "/downloads/v0.1.1/F1-Stalker-0.1.1-macos-universal.dmg",
    fileName: "F1-Stalker-0.1.1-macos-universal.dmg",
  },
  {
    label: "macOS ARM64",
    href: "/downloads/v0.1.1/F1-Stalker-0.1.1-macos-arm64.dmg",
    fileName: "F1-Stalker-0.1.1-macos-arm64.dmg",
  },
  {
    label: "Linux AMD64",
    href: "/downloads/v0.1.1/F1-Stalker-0.1.1-linux-amd64.tar.gz",
    fileName: "F1-Stalker-0.1.1-linux-amd64.tar.gz",
  },
  {
    label: "Windows 10/11 AMD64",
    href: "/downloads/v0.1.1/F1-Stalker-0.1.1-windows-amd64.zip",
    fileName: "F1-Stalker-0.1.1-windows-amd64.zip",
  },
]
