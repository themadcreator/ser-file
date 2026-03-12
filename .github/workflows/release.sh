#!/bin/bash

if [[ -z "$VER" ]]; then
    echo 'The environment variable VER is not set. Aborting.'
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    echo "Git working directory is dirty. Aborting."
    exit 1
fi

cargo set-version $VER
git add Cargo.toml Cargo.lock
git commit -m "Bump version ${VER}"

echo "Releasing version ${VER}"
echo "Press [ENTER] when ready..."
read -n 1

git push
git tag -a "v${VER}" -m "Release version ${VER}"
git push origin --tags
