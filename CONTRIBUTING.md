# Contributing to Formato

First off, thank you for considering contributing to Formato! It's people like you who make open-source tools better for everyone.

To ensure a smooth collaboration, please take a moment to review the following guidelines.

## Code Style & Guidelines

Consistency is key to keeping the codebase maintainable. Since Formato is built with **SvelteKit + TypeScript** (frontend) and **Rust** (backend via Tauri), we follow these principles:

### Frontend (Svelte / TypeScript)
- **Language**: Modern TypeScript. Avoid `any` where possible; use proper type definitions.
- **Naming Conventions**:
  - `camelCase` for variables and functions.
  - `PascalCase` for Svelte components and classes.
- **Components**: Keep components small and focused. Use `$props` and `$state` (Svelte 5 runes) instead of legacy `export let`.
- **Styling**: Use Tailwind CSS. Avoid inline styles unless absolutely necessary.
- **i18n**: All user-facing strings **must** be added to the `messages/` directory via Paraglide. Do not hardcode strings in components.

### Backend (Rust)
- **Language**: Modern Rust (2021 edition). Use `cargo fmt` and `cargo clippy` before submitting.
- **Naming Conventions**:
  - `snake_case` for variables and functions.
  - `PascalCase` for structs and enums.
- **Error Handling**: Use `Result<T, E>` and custom error types. Avoid `unwrap()` and `expect()` in production code (exceptions: `fn main()` or initialization paths).
- **Modules**: Keep modules focused. New conversion logic should go into `src-tauri/src/convert/` with a dedicated file per format or type.

### Clean PRs
- Ensure your code is linted and free of commented-out blocks or `console.log` statements before submitting.
- Remove unused dependencies and imports.
- Keep changes focused. If you have multiple unrelated changes, split them into separate PRs.

## Workflow

We use GitHub to track issues and merge changes. Here is the best way to get your contribution through:

### Reporting Bugs & Opening Issues

- **Search First**: Check if the issue or feature request has already been reported. We highly encourage reading our `README.md` and `docs/` before opening an issue.
- **Be Specific**: Provide your OS version, Formato version (from `Cargo.toml` or `package.json`), and steps to reproduce the bug.
- **Logs**: If the app crashes, run it from the terminal using `npm run tauri dev` and attach the output.
- **Screenshots**: If the issue is visual, attach a screenshot.

### Submitting Features or Fixes

- **Fork & Branch**: Create a descriptively named branch (e.g., `fix/caching-bug` or `feat/audio-conversion`).
- **Commit Messages**: Use clear, imperative messages (e.g., "Add support for OPDS catalogs" instead of "I fixed some stuff").
- **One Change per PR**: Keep Pull Requests focused. If you have two unrelated fixes, please open two separate PRs.
- **Pre-PR Checklist**:
  - [ ] Run `npm run check` (TypeScript + Svelte checks)
  - [ ] Run `npm run test` (Vitest tests)
  - [ ] Run `cargo fmt --check` (Rust formatting)
  - [ ] Run `cargo clippy` (Rust linting)
  - [ ] Manually test the feature/fix in the UI

## Translation Guide

Formato aims to be accessible to everyone, regardless of the language they speak.

- **Platform**: Localization is handled via `@inlang/paraglide-js`. All strings live in `src/lib/paraglide/messages/`.
- **How to help**:
  - Check the `messages/` folder for your language's file (e.g., `ru.json` or `en.json`).
  - If it doesn't exist, you can initialize a new one from the `en.json` template.
  - Ensure that technical terms (like "Converter", "Metadata", "Archive") are translated consistently across the app.
- **Testing**: You can test your translations locally by running `npm run dev` and switching languages in the settings.

## Getting Started

If you are setting up your environment for the first time, make sure you have the following dependencies installed:

- `Node.js` (v20+)
- `Rust` (stable)
- `Tauri CLI`

> **Note**: For a detailed setup of the development environment, please refer to the `README.md` or our build documentation.

### Prerequisites

In order to not waste your time implementing a change that has already been declined, or is generally not needed, start by [opening an issue](https://github.com/App-Lab-Hub/formato/issues/new/choose) describing the problem you would like to solve.

For the best experience to build Formato for yourself, use a recent version of Node.js and Rust. Refer to the [Tauri documentation](https://v2.tauri.app/start/prerequisites/) for details on setting up the development environment prerequisites on different platforms.

Basically you need to install or update the following development tools:

- **Node.js** and **npm** for SvelteKit development
- **Rust** and **Cargo** for Tauri development

```bash
nvm install v20
nvm use v20
rustup update
```

## Getting Started with Development

To get started with Formato, follow these steps to clone and build the project.

### 1. Clone the Repository

```bash
git clone https://github.com/App-Lab-Hub/formato.git
cd formato
```
### 2. Install Dependencies

```bash
npm install
```

### 3. Build for Development

```bash
npm run dev          # Start the Vite dev server (frontend only)
npm run tauri dev    # Start the Tauri desktop app (full app)
```

### 4. Build for Production

```bash
npm run tauri build
```

### 5. Run Tests
```bash
npm run test         # Run Vitest tests
npm run check        # Run TypeScript and Svelte checks
```

## Project Structure

Understanding the project structure helps you navigate the codebase quickly:

formato/
├── src/                     # SvelteKit frontend
│   ├── lib/
│   │   ├── components/      # Reusable Svelte components
│   │   ├── data/            # Static data (formats, models, settings)
│   │   ├── stores/          # Svelte stores (app state, loader)
│   │   ├── styles/          # Global CSS
│   │   ├── types/           # TypeScript types
│   │   └── utils/           # Utility functions
│   └── routes/              # SvelteKit routes (pages)
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── convert/         # Conversion logic (audio, video, docx, etc.)
│   │   ├── db/              # Database models and logic (SQLite)
│   │   ├── utils/           # Rust utilities
│   │   └── paths.rs         # Path management for app data
│   └── Cargo.toml           # Rust dependencies
├── static/                  # Static assets
├── data/                    # Screenshots, icons, etc.
├── messages/                # i18n files (via Paraglide)
├── package.json             # Frontend dependencies and scripts
└── README.md                # Project documentation


## Implement Your Changes

This project uses SvelteKit + Tauri. The code for the frontend is in `src/`, and the Rust backend is in `src-tauri/`.

| Command          | Description                                        |
| ---------------- | -------------------------------------------------- |
| `npm run dev`    | Starts the development server for the web app only |
| `npm run check`  | Runs TypeScript and Svelte checks                  |
| `npm run test`   | Runs Vitest tests                                  |
| `cargo fmt`      | Formats Rust code                                  |
| `cargo clippy`   | Lints Rust code                                    |

### Editor-specific setup

#### VS Code

Upon opening the project, you will be prompted to install the following recommended extensions:

- Svelte for VS Code (`svelte.svelte-vscode`)
- rust-analyzer (`rust-lang.rust-analyzer`)
- Tailwind CSS IntelliSense (`bradlc.vscode-tailwindcss`)
- Prettier (`esbenp.prettier-vscode`)

### When you're done

Check that your code follows the project's style guidelines by running:

```bash
npm run check
npm run test
cargo fmt --check
cargo clippy
```

Please also make a manual, functional test of your changes. When all that's done, it's time to file a pull request to upstream and fill out the title and body appropriately.

## Thank You

Thank you for taking the time to contribute to Formato! Your efforts help make this tool better for everyone. We appreciate your support and look forward to seeing your contributions.





