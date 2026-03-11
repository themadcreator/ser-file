# Set version
VER="0.1.0-pre"
cargo set-version $VER

# Push current version tag
```bash
    VER="0.1.0-pre3"
    cargo set-version $VER
    git add Cargo.toml Cargo.lock
    git commit -m "Bump version ${VER}"
    ./.github/workflows/release.sh
```
