Bundled FFmpeg binaries are staged here during CI release builds.

Expected layout:

- `windows/ffmpeg.exe`
- `windows/ffprobe.exe`
- `macos/ffmpeg`
- `macos/ffprobe`

The desktop app prefers these bundled binaries in packaged builds and falls back to the system `PATH` during local development.
