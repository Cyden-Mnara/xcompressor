# Contributing to xcompressor

Thanks for helping improve xcompressor. This project aims to be practical, respectful of upstream open source work, and useful for people who need reliable local media compression.

## Ways to contribute

- Report bugs with your operating system, app version, source media type, selected mode, preset, target format, and the error message you saw.
- Suggest improvements that make compression, conversion, GIF creation, or batch handling clearer.
- Improve documentation, onboarding copy, translations, and release notes.
- Submit focused pull requests that solve one problem at a time.
- Test builds on Windows, macOS, and Linux.

## Development

Install dependencies and run the app from the repository root:

```bash
pnpm install
pnpm dev
```

Useful checks:

```bash
pnpm check
pnpm lint
pnpm rustcheck
```

## Pull request expectations

- Keep changes scoped to the issue or improvement being addressed.
- Preserve existing licenses and attribution for open source dependencies.
- Add or update tests when behavior changes.
- Include screenshots for visible UI changes.
- Explain user-facing behavior changes in the pull request description.
- Avoid committing generated build artifacts unless they are intentionally part of the release process.

## Contribution ethics

- Be honest about authorship, licenses, limitations, and tradeoffs.
- Do not submit code, media, or documentation that you do not have permission to contribute.
- Credit upstream projects and link to prior work when a change is derived from it.
- Prefer fixes that can be shared upstream when a problem belongs in a dependency.
- Do not add telemetry, network calls, or data collection without a clear user benefit and explicit review.
- Treat user media as private. Do not ask for personal media samples publicly; request minimal reproducible examples where possible.

## Review conduct

Reviews should focus on correctness, maintainability, usability, accessibility, security, and respect for upstream licenses. Disagreement is expected, but keep it specific and technical.
