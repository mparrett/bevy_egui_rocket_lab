# Feature: Host WASM Build on GitHub Pages

## Status: DONE (2026-03)

Game is live at `mparrett.github.io/bevy_egui_rocket_lab`. Auto-deploys on push to main.

## What was done

1. Created `.github/workflows/deploy-pages.yml` — builds WASM, runs wasm-opt, deploys via actions/deploy-pages
2. Enabled GitHub Pages in repo settings (Source: GitHub Actions)
3. Added wasm-opt -Oz step (~100MB → ~33MB)
4. Added link to live demo in README

## Implementation details

- Workflow installs Rust stable + wasm32-unknown-unknown target
- Pins wasm-bindgen-cli to 0.2.114 (must match Cargo.lock)
- Downloads binaryen version_121 for wasm-opt
- Deploys: index.html + out/ + assets/audio/ + assets/textures/ + assets/fonts/
- Uses Swatinem/rust-cache for build caching
