#!/bin/sh
# audit.sh — local one-shot equivalent of CI's quality gates.
# Run before pushing. Mirrors `.github/workflows/ci.yml` so anything
# that passes here passes in CI (modulo aarch64 cross-build which
# requires cc5_aarch64 in the toolchain bundle).
#
# Usage: ./scripts/audit.sh
# Exits non-zero on the first failed gate.

set -e

# Run from project root — every other path is relative.
cd "$(dirname "$0")/.."

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
gate()  { printf '\n--- %s ---\n' "$*"; }

gate "deps"
cyrius deps

gate "syntax"
for f in src/*.cyr; do
    cyrius check --with-deps "$f"
done

gate "format"
fail=0
for f in src/*.cyr tests/*.tcyr tests/*.bcyr tests/*.fcyr; do
    [ -f "$f" ] || continue
    if ! diff -q <(cyrius fmt "$f" 2>/dev/null) "$f" > /dev/null; then
        red "needs fmt: $f"
        fail=1
    fi
done
[ "$fail" -eq 0 ] || { red "fmt drift"; exit 1; }
green "fmt clean"

gate "lint"
fail=0
for f in src/*.cyr tests/*.tcyr tests/*.bcyr tests/*.fcyr; do
    [ -f "$f" ] || continue
    out=$(cyrius lint "$f" 2>&1)
    if echo "$out" | grep -qE '^\s*warn '; then
        red "$f:"
        echo "$out" | grep '^\s*warn '
        fail=1
    fi
done
[ "$fail" -eq 0 ] || { red "lint warnings"; exit 1; }
green "lint clean"

gate "vet (include-graph audit)"
cyrius vet src/main.cyr > /dev/null
green "vet ok"

gate "build (DCE)"
mkdir -p build
CYRIUS_DCE=1 cyrius build src/main.cyr build/aegis 2>&1 | tail -1
size=$(wc -c < build/aegis)
green "aegis: ${size} bytes"

gate "ELF check"
xxd -l 4 build/aegis | grep -q "7f45 4c46" && green "ELF magic ok" \
    || { red "build/aegis is not an ELF"; exit 1; }

gate "smoke"
out=$(./build/aegis 2>&1)
echo "$out" | grep -q "aegis ready" && green "smoke ok" \
    || { red "smoke output: $out"; exit 1; }

gate "tests"
cyrius test tests/aegis.tcyr 2>&1 | tail -2

gate "fuzz"
for f in tests/*.fcyr; do
    [ -f "$f" ] || continue
    name=$(basename "$f" .fcyr)
    CYRIUS_DCE=1 cyrius build "$f" "build/$name" 2>&1 | tail -1
    timeout 10 "build/$name" || { red "FUZZ CRASH: $name"; exit 1; }
done

gate "bench"
for b in tests/*.bcyr; do
    [ -f "$b" ] || continue
    name=$(basename "$b" .bcyr)
    CYRIUS_DCE=1 cyrius build "$b" "build/$name" 2>&1 | tail -1
    "build/$name"
done

gate "security pattern scan"
fail=0
scan() {
    label="$1"; pat="$2"
    hits=$(grep -rn "$pat" src/ 2>/dev/null | awk -F: '
        { line=$3; for(i=4;i<=NF;i++) line=line":"$i;
          sub(/^[[:space:]]+/,"",line);
          if (substr(line,1,1) != "#") print $1":"$2":"line }')
    if [ -n "$hits" ]; then
        red "FAIL: $label"
        echo "$hits"
        fail=1
    fi
}
scan "raw execve"     'syscall( *59'
scan "shadow access"  '"/etc/shadow"'
scan "writes to /bin"  '"/bin/'
scan "writes to /sbin" '"/sbin/'

awk -F: '
    /var [A-Za-z_][A-Za-z0-9_]*\[[0-9]+\]/ {
        match($0, /\[[0-9]+\]/); s=substr($0,RSTART+1,RLENGTH-2); n=s+0;
        if (n >= 65536) print FILENAME":"NR": FAIL large static buffer ("n" bytes): "$0
        else if (n >= 4096) print FILENAME":"NR": warn fn-scope buffer ("n" bytes): "$0
    }' src/*.cyr | tee /tmp/aegis_bigbuf.txt
if grep -q "^.*: FAIL " /tmp/aegis_bigbuf.txt; then fail=1; fi
rm -f /tmp/aegis_bigbuf.txt

[ "$fail" -eq 0 ] && green "security scan clean" || { red "security scan FAIL"; exit 1; }

gate "docs"
missing=0
for d in README.md CHANGELOG.md VERSION CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md LICENSE cyrius.cyml; do
    [ -f "$d" ] && green "  OK: $d" || { red "  MISSING: $d"; missing=1; }
done
[ "$missing" -eq 0 ] || exit 1

v=$(cat VERSION | tr -d '[:space:]')
grep -q "$v" CHANGELOG.md || { red "FAIL: version $v not in CHANGELOG.md"; exit 1; }
grep -q '^version = "\${file:VERSION}"' cyrius.cyml || {
    red "FAIL: cyrius.cyml version must be \${file:VERSION}"; exit 1; }
green "  version $v consistent"

printf '\n'
green "=== aegis audit: PASSED ==="
