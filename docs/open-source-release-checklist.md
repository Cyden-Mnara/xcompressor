# Open Source Release Checklist

This checklist is for publishing xcompressor as an open source project. It is not legal advice.

## Project license

- Choose a license for xcompressor itself before publishing.
- Add a root `LICENSE` file.
- Make sure every dependency license is compatible with the way the app is distributed.
- Keep third-party notices visible in the repository and in release artifacts where required.
- If you use an EULA or end-user notice, make sure it does not override open source licenses or restrict rights required by those licenses.

## FFmpeg release compliance

xcompressor uses FFmpeg as an external command-line tool. Release builds may bundle `ffmpeg` and `ffprobe`, so every public download must be matched with the exact FFmpeg source and license information for those binaries.

Before publishing a release that bundles FFmpeg:

- Use FFmpeg binaries built without `--enable-gpl` and without `--enable-nonfree` if you want the simpler LGPL path.
- Keep the exact FFmpeg version, configure line, build source, and build date.
- Keep a `changes.diff` file for any FFmpeg source changes. If there are no changes, include an empty file or a short note saying the source was not modified.
- Publish the matching FFmpeg source archive next to the app downloads.
- Include FFmpeg license notices in the release notes and app credits.
- Add the required FFmpeg notice anywhere users download the app.
- Do not rename FFmpeg binaries in a misleading way.
- Check licenses for codecs and libraries compiled into FFmpeg, such as LAME, libopus, libvpx, or others.
- Do not bundle GPL-enabled FFmpeg builds unless you are prepared for the GPL obligations that may apply to the distributed app.

Recommended release artifact layout:

```text
release/
  xcompressor-<version>-<platform>.<installer>
  ffmpeg-source-<ffmpeg-version>.zip
  ffmpeg-build-info.txt
  ffmpeg-changes.diff
  THIRD_PARTY_NOTICES.md
```

Suggested notice for download pages:

```html
This software uses code of <a href="https://ffmpeg.org">FFmpeg</a>
licensed under the <a href="https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html">LGPLv2.1</a>.
The matching FFmpeg source for the bundled binaries can be downloaded from this release page.
```

## Financial support policy

Financial support is not currently enabled. Do not publish donation or sponsorship links until there is a working payout method.

If donations or sponsorships are added later:

- Say that support is optional.
- Do not gate source code, FFmpeg source, license notices, or required compliance materials behind payment.
- Do not imply that donating grants a commercial FFmpeg license.
- Do not promise codec or patent rights.
- Keep paid perks limited to project updates, acknowledgements, or similar low-risk community benefits.
- Track donation income according to local tax rules.

## Privacy and user media

- Do not upload user media without explicit consent.
- Ask for small synthetic reproduction files when debugging.
- Avoid collecting telemetry unless there is a clear user benefit and an opt-in design.

## Before announcing

- Root `LICENSE` exists.
- End-user notice or EULA has been reviewed, if you plan to ship one.
- `README.md`, `CONTRIBUTING.md`, and `CODE_OF_CONDUCT.md` are current.
- Credits page mentions FFmpeg and open source dependencies.
- Support page clearly explains the currently available support options.
- Download pages include FFmpeg notices and matching FFmpeg source links.
- Release artifacts include third-party notices.
