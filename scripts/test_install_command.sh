#!/usr/bin/env bash
# Proves the published install command (templates.INSTALL_COMMAND) can never
# alter, overwrite, or merge into anything a user already has under
# ~/.claude/skills/ that this repo doesn't own — a user asked "won't this
# alter other skills?" and this is the actual answer, re-checked on every
# run rather than trusted from memory.
#
# Four scenarios, each in its own throwaway HOME so a real ~/.claude is
# never touched:
#   A. fresh install                        — clones cleanly
#   B. re-running once already installed     — updates in place (git pull),
#                                               does not error
#   C. an empty dir already named TBaguette  — clones into it cleanly
#   D. a NON-empty, non-git dir already named
#      TBaguette (the real collision case)   — refuses, leaves it untouched
#
# Sibling directories (other skills, other plugins) are populated in every
# scenario and checksummed before/after to prove they are never touched,
# not just assumed to be.
#
# Usage:
#   bash scripts/test_install_command.sh

set -u

REPO_URL="https://github.com/LeSplooch/tbaguette-skills.git"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

checks=0
fails=0

check() {
    checks=$((checks + 1))
    if [ "$1" = "0" ]; then
        echo "  ok  $2"
    else
        echo "  FAIL  $2"
        fails=$((fails + 1))
    fi
}

install_cmd() {
    # Exactly templates.INSTALL_COMMAND — kept in sync by eye since bash
    # can't import the Python constant; test_templates.py separately
    # asserts this exact string appears verbatim on the rendered page, so a
    # drift between the two would fail there first.
    HOME="$1" bash -c \
        '[ -d ~/.claude/skills/TBaguette/.git ] && git -C ~/.claude/skills/TBaguette pull || git clone '"$REPO_URL"' ~/.claude/skills/TBaguette'
}

seed_sibling_skills() {
    local home="$1"
    mkdir -p "$home/.claude/skills/some-other-skill"
    mkdir -p "$home/.claude/skills/another-plugin/.claude-plugin"
    mkdir -p "$home/.claude/skills/another-plugin/skills/foo"
    echo "precious user content that must survive" > "$home/.claude/skills/some-other-skill/SKILL.md"
    echo '{"name": "another-plugin"}' > "$home/.claude/skills/another-plugin/.claude-plugin/plugin.json"
    echo "more precious user content" > "$home/.claude/skills/another-plugin/skills/foo/SKILL.md"
}

siblings_checksum() {
    find "$1/.claude/skills/some-other-skill" "$1/.claude/skills/another-plugin" -type f \
        -exec sha256sum {} \; | sort
}

echo "scenario A: fresh install"
home_a="$WORKDIR/a"
mkdir -p "$home_a"
seed_sibling_skills "$home_a"
before_a="$(siblings_checksum "$home_a")"
install_cmd "$home_a" >/dev/null 2>&1
check "$?" "clone exits 0"
check "$([ -f "$home_a/.claude/skills/TBaguette/README.md" ] && echo 0 || echo 1)" "TBaguette content actually present"
check "$([ "$before_a" = "$(siblings_checksum "$home_a")" ] && echo 0 || echo 1)" "sibling skills byte-identical after install"

echo "scenario B: re-run when already installed (must update, not error)"
home_b="$home_a"  # continues from A, which already installed TBaguette
before_b="$(siblings_checksum "$home_b")"
out_b="$(install_cmd "$home_b" 2>&1)"
rc_b=$?
check "$rc_b" "re-run exits 0 (does not error)"
check "$(echo "$out_b" | grep -qE "Already up to date|Updating|Fast-forward" && echo 0 || echo 1)" "re-run reports a pull, not a clone-refused error"
check "$([ "$before_b" = "$(siblings_checksum "$home_b")" ] && echo 0 || echo 1)" "sibling skills still byte-identical after re-run"

echo "scenario C: an empty pre-existing TBaguette directory"
home_c="$WORKDIR/c"
mkdir -p "$home_c/.claude/skills/TBaguette"
seed_sibling_skills "$home_c"
before_c="$(siblings_checksum "$home_c")"
install_cmd "$home_c" >/dev/null 2>&1
check "$?" "clone into empty existing dir exits 0"
check "$([ -f "$home_c/.claude/skills/TBaguette/README.md" ] && echo 0 || echo 1)" "TBaguette content actually present"
check "$([ "$before_c" = "$(siblings_checksum "$home_c")" ] && echo 0 || echo 1)" "sibling skills byte-identical"

echo "scenario D: a non-empty, non-git TBaguette directory (real collision)"
home_d="$WORKDIR/d"
mkdir -p "$home_d/.claude/skills/TBaguette"
echo "unrelated content some other tool put here" > "$home_d/.claude/skills/TBaguette/dont-touch-me.txt"
seed_sibling_skills "$home_d"
before_marker="$(sha256sum "$home_d/.claude/skills/TBaguette/dont-touch-me.txt")"
before_d="$(siblings_checksum "$home_d")"
install_cmd "$home_d" >/dev/null 2>&1
check "$([ $? -ne 0 ] && echo 0 || echo 1)" "refuses (nonzero exit) rather than merging into unrelated content"
check "$([ "$before_marker" = "$(sha256sum "$home_d/.claude/skills/TBaguette/dont-touch-me.txt")" ] && echo 0 || echo 1)" "the colliding directory's own content is untouched"
check "$([ "$before_d" = "$(siblings_checksum "$home_d")" ] && echo 0 || echo 1)" "sibling skills byte-identical even in the refusal case"

echo
if [ "$fails" -eq 0 ]; then
    echo "$checks checks passed."
    exit 0
else
    echo "$fails of $checks checks FAILED."
    exit 1
fi
