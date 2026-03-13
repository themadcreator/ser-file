#!/bin/bash

if [ -n "$(git status --porcelain)" ]; then
    echo "Git working directory is dirty. Aborting."
    exit 1
fi

ver=$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')

echo -e "\n\nCurrent version is $ver\n\n"

read -p "What to bump? (patch/rc) " bump
cargo set-version --dry-run --bump $bump

echo ""
read -p "Proceed with this change and release? (y/n) " yn
case $yn in
    [Yy]* ) echo "Confirmed."; break;; # Handles 'y', 'Y', 'yes', 'Yes', etc.
    * ) exit;;
esac

cargo set-version --bump $bump
ver=$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')

echo "Releasing version ${ver}"

git add Cargo.toml Cargo.lock
git commit -m "Bump version ${ver}"
git push

git tag -a "v${ver}" -m "Release version ${ver}"
git push origin --tags
