# Set version
VER="0.1.0-pre"
cargo set-version $VER

# Push current version tag
```bash
if [ -n "$(git status --porcelain)" ]; then
    echo "Git working directory is dirty. Aborting tag creation."
else
    VER=$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')
    echo "Releasing version ${VER}"
    echo "Press [ENTER] when ready..."
    read  -n 1 
    git tag -a "v${VER}" -m "Release version ${VER}"
    git push origin --tags
fi
```