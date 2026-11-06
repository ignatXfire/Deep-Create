# Working with the public copy

Use a separate test directory and synthetic data. This is experimental software with documented defects. Compilation and ordinary tests do not require model-provider credentials.

## Environment

- Windows for native components.
- Rust as specified by `rust-toolchain.toml` and Cargo manifests.
- MSVC Build Tools and Windows SDK for native compilation.
- WebView2 for the desktop application.
- Node `>=22.12.0 <23` and pnpm `12.3.4`, as pinned in the historical `package.json`. Availability of those exact versions in public registries was not independently established during packaging.

Run Cargo in a Developer PowerShell with the MSVC linker available. The public `.cargo/config.toml` uses a local `target` directory.

```powershell
node tools/check-architecture.mjs
cargo test --locked -p dc-domain -p dc-protocol -p dc-store
pnpm install --frozen-lockfile
pnpm run ui:build
cargo test --locked --workspace
pnpm run desktop:dev
```

Tauri macros may require `ui/dist` to exist. Historical `Enter-BuildEnvironment`, `build-env`, and worktree helpers may refer to tools omitted from this public snapshot. Treat those anonymized paths as historical environment references rather than installation instructions.

## Model connections

The public `tools/external-model.mjs` reads only `OMNIROUTE_API_KEY` from its process environment. Direct access to the author's credential database and selection by a private key ID were removed. Configure your own compatible local endpoint if you intend to study this helper. Never put a real key in job JSON, documentation, or source code.

Historical Dash model examples do not guarantee route availability. Read `Verify.ps1` before using it: it is a verification utility from its own development stage, not proof that every documented capability is covered.
