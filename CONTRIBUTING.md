# Contributing to DrawOver

First off — thank you for considering contributing to DrawOver! 🎨

This document covers everything you need to get started as a contributor.

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| **macOS** | 12+ (Monterary or later) | Required for building and testing (DrawOver is macOS-only) |
| **Rust** | stable | Install via [rustup](https://rustup.rs/) |
| **bun** | latest | Install via [bun.sh](https://bun.sh/) |
| **Xcode Command Line Tools** | latest | `xcode-select --install` |

> **Note:** While you can browse and edit the code on any OS, building and testing DrawOver requires macOS because it relies on macOS-specific screen capture and overlay APIs.

## Development Setup

```bash
# Clone the repository
git clone https://github.com/codecoradev/drawover.git
cd drawover

# Install frontend dependencies
bun install

# Start the dev server (hot reload for both frontend and Rust)
bun run tauri dev
```

That's it! The app should launch with hot module replacement enabled.

## Project Structure

```
drawover/
├── src/              # Svelte 5 frontend (UI, toolbar, canvas state)
├── src-tauri/        # Rust backend (Tauri 2, drawing engine, screen overlay)
│   └── src/          # Rust source code
├── package.json      # Frontend dependencies
└── bun.lockb         # Lock file
```

## CLA Signing

Before we can accept your pull request, you must sign our [Contributor License Agreement](CLA.md). This is a one-time requirement.

**To sign:**
1. Read the [CLA](CLA.md)
2. Add a signature file to `.github/cla-signatures/` named after your GitHub username
3. Open a PR for your signature — once merged, the CLA check passes automatically on all future PRs

The CLA check runs automatically on every pull request. If you haven't signed, the CLA bot will leave a comment with instructions.

## Code Style

### Rust
- **Formatting:** `cargo fmt` — must be applied before committing
- **Linting:** `cargo clippy -- -D warnings` — zero warnings allowed
- **Tests:** Write unit tests for core drawing logic in `src-tauri/src/`

### Frontend (Svelte 5 + TypeScript)
- **Type checking:** `bun run check` — must pass (uses `svelte-check`)
- **Formatting:** Follow existing style (2-space indentation, no semicolons)
- Use Svelte 5 runes (`$state`, `$derived`, `$effect`) — not Svelte 4 stores

### Checking everything before pushing

```bash
# Rust
cargo fmt
cargo clippy -- -D warnings
cargo test

# Frontend
bun run check
```

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | When to use |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Code style (formatting, no logic change) |
| `refactor` | Code restructuring, no behavior change |
| `perf` | Performance improvement |
| `test` | Adding or fixing tests |
| `chore` | Build tooling, dependencies, CI |

### Examples

```
feat(pen): add pressure-sensitive pen width
fix(eraser): correct stroke deletion on undo
docs(readme): update roadmap section
chore(deps): bump tauri to 2.1.0
```

## Pull Request Process

1. **Create a branch** from `main`: `git checkout -b feat/my-feature`
2. **Make your changes** and ensure all checks pass:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   bun run check
   ```
3. **Write a clear PR description** using the pull request template
4. **Reference any issues** your PR addresses (e.g., `Closes #42`)
5. **Sign the CLA** if you haven't already
6. **Request review** — a maintainer will review your PR

### PR Checklist

- [ ] Code follows the style guidelines (fmt + clippy + check)
- [ ] Self-reviewed the code
- [ ] Added or updated tests for changes
- [ ] Updated documentation if needed
- [ ] CLA signed
- [ ] Commit messages follow conventional commits

## Reporting Bugs

Use the [Bug Report template](https://github.com/codecoradev/drawover/issues/new?template=bug_report.md) when opening an issue. Include:

- macOS version and Mac model (Intel or Apple Silicon)
- Steps to reproduce
- Expected vs actual behavior
- Screenshots or screen recordings if applicable

## Requesting Features

Use the [Feature Request template](https://github.com/codecoradev/drawover/issues/new?template=feature_request.md). Check the [roadmap](README.md#roadmap) first — your idea might already be planned!

## Code of Conduct

Be kind and respectful. We're building this together. Personal attacks, harassment, or discrimination of any kind will not be tolerated.

---

Questions? [Open a discussion](https://github.com/codecoradev/drawover/discussions) — we're happy to help!

Built with ❤️ by [CodeCoraDev](https://github.com/codecoradev).
