# m2s2-cli

[![GitHub Sponsors](https://img.shields.io/github/sponsors/mgmaster24?style=flat&logo=githubsponsors&label=Sponsor)](https://github.com/sponsors/mgmaster24)

The official CLI for scaffolding and working with [M²S²](https://github.com/M2S2-Engineering-Group) design system projects. Create new React, Angular, or Vue applications pre-wired with M²S² components, generate components, and keep your installation up to date — all from the terminal.

## Table of Contents

- [Installation](#installation)
- [Commands](#commands)
  - [new](#m2s2-new)
  - [generate component](#m2s2-generate-component)
  - [upgrade](#m2s2-upgrade)
  - [completions](#m2s2-completions)
- [Building from Source](#building-from-source)
- [Testing](#testing)
- [Project Structure](#project-structure)
- [Workflows](#workflows)
- [Contributing](#contributing)

---

## Installation

### Shell installer (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/M2S2-Engineering-Group/m2s2-cli/releases/latest/download/m2s2-cli-installer.sh | sh
```

### PowerShell (Windows)

```powershell
irm https://github.com/M2S2-Engineering-Group/m2s2-cli/releases/latest/download/m2s2-cli-installer.ps1 | iex
```

### npm / npx

```bash
# Install globally
npm install -g @m2s2/cli

# Or run without installing
npx @m2s2/cli new my-app
```

### Cargo

```bash
cargo install m2s2-cli
```

### Verify installation

```bash
m2s2 --version
```

---

## Commands

### `m2s2 new`

Scaffold a new project pre-configured with the M²S² design system.

```bash
m2s2 new <name> [OPTIONS]
```

**Arguments**

| Argument | Description |
|----------|-------------|
| `<name>` | Project directory name |

**Options**

| Flag | Description |
|------|-------------|
| `--framework <react\|angular\|vue>` | Framework to scaffold. Prompted interactively if omitted. |
| `--skip-install` | Skip running `npm install` after writing project files. |

**Examples**

```bash
# Interactive framework selection
m2s2 new my-app

# Explicit framework
m2s2 new my-app --framework react
m2s2 new my-app --framework angular
m2s2 new my-app --framework vue

# Skip npm install (useful in CI or offline environments)
m2s2 new my-app --framework react --skip-install
```

**What gets generated**

*React*
```
my-app/
├── src/
│   ├── main.tsx        # BrowserRouter + M2S2Provider
│   ├── App.tsx         # Navbar + Footer wired with M²S² configs
│   └── App.scss
├── index.html
├── package.json        # @m2s2/react-lib, @m2s2/tokens, Vite, TypeScript
├── vite.config.ts
├── tsconfig.json
└── .gitignore
```

*Angular*
```
my-app/
├── src/
│   ├── main.ts
│   ├── styles.scss
│   ├── index.html
│   └── app/
│       ├── app.component.ts    # Standalone, imports NavbarComponent + FooterComponent
│       ├── app.component.html
│       ├── app.component.scss
│       ├── app.routes.ts
│       └── app.config.ts
├── package.json                # @m2s2/ng-lib, @m2s2/tokens, Angular 21
├── angular.json
├── tsconfig.json
├── tsconfig.app.json
└── .gitignore
```

*Vue*
```
my-app/
├── src/
│   ├── main.ts         # createApp with @m2s2/vue-lib styles
│   ├── App.vue         # Navbar + Footer wired with M²S² configs
│   └── App.scss
├── index.html
├── package.json        # @m2s2/vue-lib, @m2s2/tokens, Vite, TypeScript
├── vite.config.ts
├── tsconfig.json
└── .gitignore
```

---

### `m2s2 generate component`

Scaffold a new component inside an existing M²S² project.

```bash
m2s2 generate component <name> [OPTIONS]
```

Run this command from your project root. The framework is detected automatically by reading `package.json` — no flag needed if your project was created with `m2s2 new`.

**Arguments**

| Argument | Description |
|----------|-------------|
| `<name>` | Component name. Accepts any casing — `MyCard`, `my-card`, and `myCard` all produce the same output. |

**Options**

| Flag | Description |
|------|-------------|
| `--framework <react\|angular\|vue>` | Override framework detection. |
| `--path <dir>` | Override the output directory. A subdirectory named after the component is always created inside `<dir>`. |

**Examples**

```bash
# Auto-detect framework from package.json
m2s2 generate component HeroSection

# Kebab-case input — same result
m2s2 generate component hero-section

# Override output directory
m2s2 generate component HeroSection --path src/features

# Override framework
m2s2 generate component HeroSection --framework vue
```

**What gets generated**

*React* — written to `src/components/<Name>/`
```
src/components/HeroSection/
├── HeroSection.tsx     # Typed props interface, BEM className application
├── HeroSection.scss    # Scoped class stub (.hero-section)
└── index.ts            # Barrel re-export
```

*Angular* — written to `src/app/components/<name>/`
```
src/app/components/hero-section/
├── hero-section.component.ts    # Standalone component, app-hero-section selector
├── hero-section.component.html  # BEM wrapper div
└── hero-section.component.scss  # Scoped class stub (.hero-section)
```

*Vue* — written to `src/components/<Name>/`
```
src/components/HeroSection/
├── HeroSection.vue     # <script setup> SFC with slot and scoped SCSS
├── HeroSection.scss    # Scoped class stub (.hero-section)
└── index.ts            # Barrel re-export
```

---

### `m2s2 completions`

Install shell completions for `m2s2`. Auto-detects your shell from `$SHELL` and writes a completion script to your home directory, then patches your shell's rc file to source it automatically.

```bash
m2s2 completions [shell]
```

**Arguments**

| Argument | Description |
|----------|-------------|
| `[shell]` | Shell to generate completions for. Auto-detected from `$SHELL` if omitted. One of: `bash`, `zsh`, `fish`, `elvish`, `powershell`. |

**Examples**

```bash
# Auto-detect shell and install
m2s2 completions

# Explicit shell
m2s2 completions zsh
```

After running, reload your shell or source your rc file:

```bash
source ~/.zshrc   # zsh
source ~/.bashrc  # bash
# fish sources completions automatically — no reload needed
```

---

### `m2s2 upgrade`

Check for and install a newer version of the CLI.

```bash
m2s2 upgrade [OPTIONS]
```

**Options**

| Flag | Description |
|------|-------------|
| `--check` | Print available version without installing. |

**Examples**

```bash
# Check if an update is available
m2s2 upgrade --check

# Download and install the latest version
m2s2 upgrade
```

The upgrade command hits the GitHub Releases API, compares the latest release tag against the running binary version, and — if a newer version exists — runs the official installer script for your platform. Restart your terminal after upgrading.

---

## Building from Source

**Prerequisites:** Rust 1.85+ (edition 2024), Node.js 18+ for running scaffolded projects.

```bash
git clone https://github.com/M2S2-Engineering-Group/m2s2-cli.git
cd m2s2-cli

# Debug build
cargo build

# Run directly
cargo run -- new my-app --framework react --skip-install

# Optimised release build (matches the distributed binary)
cargo build --profile dist
```

The binary is written to `target/debug/m2s2` (debug) or `target/dist/m2s2` (dist profile).

### Templates

All scaffold and generate templates live under `templates/` and are embedded into the binary at compile time via [`rust-embed`](https://github.com/pyros2097/rust-embed). Changes to template files require a rebuild to take effect.

```
templates/
├── react/          # m2s2 new --framework react
├── angular/        # m2s2 new --framework angular
├── vue/            # m2s2 new --framework vue
└── generate/
    ├── react/      # m2s2 generate component (React)
    ├── angular/    # m2s2 generate component (Angular)
    └── vue/        # m2s2 generate component (Vue)
```

---

## Testing

```bash
# Run all tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

End-to-end smoke test (no npm install required):

```bash
# Scaffold a React project and inspect the output
cargo run -- new test-app --framework react --skip-install
find test-app -type f

# Generate a component inside it
cd test-app
../target/debug/m2s2 generate component MyCard
```

---

## Project Structure

```
src/
├── main.rs                        # CLI entry point, command routing
├── scaffold/
│   └── mod.rs                     # Template engine (rust-embed + Handlebars)
└── commands/
    ├── mod.rs
    ├── new.rs                     # m2s2 new
    ├── upgrade.rs                 # m2s2 upgrade
    └── generate/
        ├── mod.rs                 # m2s2 generate (subcommand router)
        └── component.rs          # m2s2 generate component
```

---

## Workflows

### CI (`ci.yml`)

Runs on every push to `main` and every pull request.

| Job | Description |
|-----|-------------|
| `build` | Compiles the crate for macOS (arm64), Linux (x86_64), and Windows (x86_64) in a matrix. |
| `test` | Runs `cargo test` on Linux. |
| `clippy` | Runs `cargo clippy -- -D warnings`. |
| `fmt` | Runs `cargo fmt --check`. |

### Release (`release-plz.yml` + `release.yml`)

Releases are fully automated from conventional commits — no PRs, no manual tagging required.

1. **CI passes on `main`** — `release-plz.yml` triggers only after the `CI` workflow succeeds.
2. **Version bump** — `release-plz update` reads conventional commit history, bumps `Cargo.toml`, and updates `CHANGELOG.md`. Only `feat`, `fix`, and `perf` commits trigger a bump; `chore`, `docs`, `refactor`, and others are skipped.
3. **Tag pushed** — the bump is committed directly to `main` and a `vX.Y.Z` tag is pushed.
4. **`release.yml` (cargo-dist) triggers** — builds platform binaries in parallel:
   - `aarch64-apple-darwin`
   - `x86_64-apple-darwin`
   - `aarch64-unknown-linux-gnu`
   - `x86_64-unknown-linux-gnu`
   - `x86_64-pc-windows-msvc`
5. **GitHub Release created** — all binaries, checksums, shell installer, and PowerShell installer are attached.
6. **`publish-npm.yml` triggers** — publishes `@m2s2/cli` to npm.
7. **`publish-crates.yml` triggers** — publishes `m2s2-cli` to crates.io.

### Template Sync (`template-sync.yml`)

Keeps scaffold template dependencies in sync with the latest published M²S² library versions.

Triggers:
- **Weekly** — every Monday at 08:00 UTC.
- **`repository_dispatch`** — fired automatically by the design system CI whenever any `@m2s2` library publishes a new release.

When a version change is detected, the workflow opens a pull request updating the pinned package versions in `templates/react/package.json.hbs`, `templates/angular/package.json.hbs`, and `templates/vue/package.json.hbs`. Merging that PR flows through the normal release pipeline.

### Required Repository Secrets

| Secret | Used By | Description |
|--------|---------|-------------|
| `APP_ID` | `release-plz.yml` | GitHub App ID for bypassing branch protection on release commits. |
| `APP_PRIVATE_KEY` | `release-plz.yml` | GitHub App private key. |
| `NPM_TOKEN` | `publish-npm.yml` | npm access token with publish rights to the `@m2s2` scope. |
| `CARGO_REGISTRY_TOKEN` | `publish-crates.yml` | crates.io API token for publishing `m2s2-cli`. |

---

## Contributing

Commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) specification — this is what release-plz uses to determine version bumps and generate the CHANGELOG.

| Prefix | Version bump |
|--------|-------------|
| `fix:` | Patch |
| `feat:` | Minor |
| `feat!:` / `BREAKING CHANGE` | Major |
| `chore:`, `docs:`, `refactor:` | No release |

```bash
# Good examples
git commit -m "feat: add m2s2 init command"
git commit -m "fix: detect camelCase component names correctly"
git commit -m "chore: update Angular template to v22"
```
