#!/bin/sh
set -eu

REPO="hodstack/hodstack"
TAG="${HOD_TAG:-}"
DIR="${HOD_INSTALL_DIR:-$HOME/.local/bin}"

if [ -n "$TAG" ]; then
    RELEASE="https://github.com/$REPO/releases/download/$TAG"
else
    RELEASE="https://github.com/$REPO/releases/latest/download"
fi

URL="${HOD_RELEASE_URL:-$RELEASE}"

fail() {
    printf 'error: %s\n' "$1" >&2
    exit 1
}

target() {
    system="$(uname -s)"
    machine="$(uname -m)"

    case "$system $machine" in
        'Darwin arm64') echo 'aarch64-apple-darwin' ;;
        'Darwin x86_64') echo 'x86_64-apple-darwin' ;;
        'Linux aarch64' | 'Linux arm64') echo 'aarch64-unknown-linux-musl' ;;
        'Linux x86_64') echo 'x86_64-unknown-linux-musl' ;;
        *) fail "no build for $system $machine" ;;
    esac
}

download() {
    if command -v curl > /dev/null 2>&1; then
        curl --fail --silent --show-error --location "$1" --output "$2"
    elif command -v wget > /dev/null 2>&1; then
        wget --quiet "$1" --output-document "$2"
    else
        fail 'this computer has no curl and no wget'
    fi
}

checksum() {
    if command -v sha256sum > /dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif command -v shasum > /dev/null 2>&1; then
        shasum --algorithm 256 "$1" | cut -d ' ' -f 1
    else
        fail 'this computer has no sha256sum and no shasum'
    fi
}

on_path() {
    case ":$PATH:" in
        *":$1:"*) return 0 ;;
        *) return 1 ;;
    esac
}

TARGET="$(target)"
FILE="hod-$TARGET.tar.gz"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

printf 'Downloading hod for %s\n' "$TARGET"
download "$URL/$FILE" "$WORK/$FILE"
download "$URL/checksums.txt" "$WORK/checksums.txt"

WANTED="$(grep " $FILE\$" "$WORK/checksums.txt" | cut -d ' ' -f 1)"
FOUND="$(checksum "$WORK/$FILE")"
[ -n "$WANTED" ] || fail "checksums.txt names no $FILE"
[ "$WANTED" = "$FOUND" ] || fail "the checksum of $FILE does not agree with checksums.txt"

tar -xzf "$WORK/$FILE" -C "$WORK" hod
mkdir -p "$DIR"
install -m 755 "$WORK/hod" "$DIR/hod" 2> /dev/null || {
    cp "$WORK/hod" "$DIR/hod"
    chmod 755 "$DIR/hod"
}

printf 'Installed %s\n' "$("$DIR/hod" --version)"
printf '         %s\n' "$DIR/hod"

if ! on_path "$DIR"; then
    printf '\nAdd it to your PATH:\n'
    printf '  export PATH="%s:$PATH"\n' "$DIR"
fi
