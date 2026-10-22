#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "Usage: $0" >&2
    exit 2
fi

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

for tool in cargo awk mktemp; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Error: required tool '$tool' is not installed." >&2
        exit 1
    fi
done

map_file=$(mktemp "${TMPDIR:-/tmp}/esp32c3-memory.XXXXXX")
trap 'rm -f -- "$map_file"' EXIT

echo "Building ESP32-C3 firmware and reporting linker memory regions..."
if cargo rustc --release --bin my_esp -- \
    -C link-arg=--print-memory-usage \
    -C "link-arg=-Map=$map_file"; then
    :
else
    status=$?
    echo "Error: build failed; any linker report above is not a valid firmware image." >&2
    exit "$status"
fi

# The linker fills the remaining DRAM region with .stack, reporting 100% usage.
awk '
function hex(value,    result, i, digit) {
    result = 0
    value = tolower(value)
    for (i = 1; i <= length(value); i++) {
        digit = index("0123456789abcdef", substr(value, i, 1)) - 1
        if (digit < 0) {
            print "Error: invalid hexadecimal address in linker map." > "/dev/stderr"
            exit 1
        }
        result = result * 16 + digit
    }
    return result
}
NF == 5 && $5 == ".rwdata_dummy" {
    dram_start = hex($1)
    have_start = 1
}
NF == 5 && $5 == ".stack" {
    stack_start = hex($1)
    stack_size = hex($3)
    have_stack = 1
}
END {
    if (!have_start || !have_stack) {
        print "Error: expected ESP32-C3 RAM sections not found in linker map." > "/dev/stderr"
        exit 1
    }
    total = stack_start + stack_size - dram_start
    used = stack_start - dram_start
    if (total <= 0 || used < 0 || stack_size < 0 || used > total) {
        print "Error: invalid RAM layout in linker map." > "/dev/stderr"
        exit 1
    }
    printf "\nMain SRAM (excluding remaining stack space):\n"
    printf "  Reserved: %.0f / %.0f bytes (%.2f%%)\n", used, total, used / total * 100
    printf "  Remaining for stack: %.0f bytes (%.2f%%)\n", stack_size, stack_size / total * 100
}
' "$map_file"

cat <<'EOF'

Notes:
  DRAM in the linker table includes the remaining stack space, hence 100%.
  IRAM and DRAM share physical SRAM; do not add their region percentages.
  dram2_seg is a separate reclaimed RAM region (including the reserved heap).
  IROM/DROM percentages use linker mapping limits, not the app partition size.
  This is static allocation, not runtime heap/stack usage or a Flash image size.
EOF
