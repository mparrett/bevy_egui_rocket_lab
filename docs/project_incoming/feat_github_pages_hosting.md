# Feature: Host WASM Build on GitHub Pages

## Summary

Deploy the WASM build to GitHub Pages so the game is playable in the browser at `mparrett.github.io/bevy_egui_rocket_lab`.

## Context

- WASM build pipeline already works (`just release-wasm` produces `out/` with `.wasm` + `.js`)
- `index.html` already exists and loads from `./out/`
- The genealogy project (`mparrett/genealogy`) already uses GitHub Pages with Jekyll — simple push-to-deploy from main branch
- Rocket lab repo is `mparrett/bevy_egui_rocket_lab`

## Approach: GitHub Actions Workflow

A GitHub Actions workflow is the cleanest option. It builds WASM on push and deploys to Pages without committing build artifacts to the repo.

### Workflow outline

1. On push to `main`, run a workflow that:
   - Installs Rust stable + `wasm32-unknown-unknown` target
   - Installs `wasm-bindgen-cli` (matching version in Cargo.lock)
   - Runs `cargo build --release --target wasm32-unknown-unknown`
   - Runs `wasm-bindgen --out-dir ./out/ --target web ...`
   - Copies `index.html` + `out/` + `assets/` to a deploy directory
   - Deploys to GitHub Pages via `actions/deploy-pages`

2. Enable Pages in repo settings → Source: GitHub Actions

### Key considerations

- **Asset path**: `index.html` loads from `./out/bevy-rocket-lab.js` — this relative path works as-is
- **Asset files**: The `assets/` directory (audio, textures, skybox) must be included in the deploy — Bevy loads these at runtime via fetch
- **Build time**: WASM release build takes ~1-2 minutes; GitHub Actions has 2000 free minutes/month
- **Artifact size**: The `.wasm` file is ~100MB unoptimized. Could add `wasm-opt` to shrink it (~50-70% reduction). GitHub Pages has a 1GB soft limit per site.
- **wasm-bindgen-cli version**: Must match the `wasm-bindgen` crate version in `Cargo.lock` (currently 0.2.114). Pin this in the workflow.

## Alternative: Commit build artifacts

Simpler but messier — run `just release-wasm` locally, commit `out/` to a `gh-pages` branch or `docs/` folder. No workflow needed but pollutes git history with large binaries.

Not recommended given the 100MB+ wasm file.

## Tasks

1. [x] Create `.github/workflows/deploy-pages.yml`
2. [ ] Enable GitHub Pages in repo settings (Source: GitHub Actions)
3. [ ] Optionally add `wasm-opt` step to reduce .wasm size
4. [ ] Test deploy and verify game loads at the Pages URL
5. [x] Add link to live demo in README

## References

- [Bevy WASM GitHub Pages guide](https://bevy-cheatbook.github.io/platforms/wasm/gh-pages.html)
- Genealogy repo uses Jekyll-based Pages (simpler case, no build step on GH side)
