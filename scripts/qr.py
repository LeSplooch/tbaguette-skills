"""QR codes for the Support dialog's crypto addresses.

Stdlib only, like the rest of the build: byte mode, error correction level M,
versions 1 to 10. That covers every address the dialog carries -- the longest,
a Cardano base address, is 103 bytes and fits version 6 -- and nothing else
calls this, so the tables stop where the need does rather than at version 40.
A longer input raises instead of silently truncating: a QR code that decodes
to part of an address is worse than no QR code.

Level M rather than L: these are read off a screen, often a phone camera
pointed at a laptop at an angle, and the extra redundancy is worth the one
version it sometimes costs. Byte mode throughout, even where alphanumeric
mode would be smaller, because most of these addresses are case-sensitive
and alphanumeric mode has no lowercase.

The construction follows ISO/IEC 18004 and the layout of Project Nayuki's
reference implementation, which is the one that is easiest to check line by
line against the standard; at a given mask, its output is bit-identical to
that implementation's. The mask is chosen by the standard's four penalty
rules, whose finder-pattern rule encoders read slightly differently, so the
chosen mask can differ from another encoder's. Any of the eight decodes; the
penalty only picks the one that scans most reliably.

    matrix(text) -> list[list[bool]]   True is a dark module
    svg_path(matrix) -> str            one <path d> for the dark modules
"""

from __future__ import annotations

# (EC codewords per block, [(block count, data codewords per block), ...]) at
# level M, ISO/IEC 18004 table 9. Short blocks come first, as they do in the
# codeword sequence.
_BLOCKS_M = {
    1: (10, [(1, 16)]),
    2: (16, [(1, 28)]),
    3: (26, [(1, 44)]),
    4: (18, [(2, 32)]),
    5: (24, [(2, 43)]),
    6: (16, [(4, 27)]),
    7: (18, [(4, 31)]),
    8: (22, [(2, 38), (2, 39)]),
    9: (22, [(3, 36), (2, 37)]),
    10: (26, [(4, 43), (1, 44)]),
}

# Centre coordinates of the alignment patterns, table E.1.
_ALIGNMENT = {
    1: [], 2: [6, 18], 3: [6, 22], 4: [6, 26], 5: [6, 30], 6: [6, 34],
    7: [6, 22, 38], 8: [6, 24, 42], 9: [6, 26, 46], 10: [6, 28, 50],
}

MAX_VERSION = max(_BLOCKS_M)
QUIET_ZONE = 4  # modules of light margin the standard requires on every side
_FORMAT_BITS_M = 0b00  # level M's two format bits


def _data_capacity(version: int) -> int:
    """Data codewords (bytes) the version holds at level M."""
    return sum(count * size for count, size in _BLOCKS_M[version][1])


def _count_bits(version: int) -> int:
    return 8 if version < 10 else 16


def _pick_version(length: int) -> int:
    for version in range(1, MAX_VERSION + 1):
        if 4 + _count_bits(version) + 8 * length <= 8 * _data_capacity(version):
            return version
    raise ValueError(
        f"{length} bytes do not fit a version-{MAX_VERSION} QR code at level M; "
        "this encoder stops there because nothing it serves is longer"
    )


# --- Reed-Solomon over GF(2^8), primitive polynomial 0x11D -----------------


def _gf_multiply(x: int, y: int) -> int:
    z = 0
    for i in reversed(range(8)):
        z = (z << 1) ^ ((z >> 7) * 0x11D)
        z ^= ((y >> i) & 1) * x
    return z


def _rs_divisor(degree: int) -> list[int]:
    """Coefficients of prod(x - a^i) for i in 0..degree-1, highest power
    first with the leading 1 dropped."""
    result = [0] * (degree - 1) + [1]
    root = 1
    for _ in range(degree):
        for j in range(degree):
            result[j] = _gf_multiply(result[j], root)
            if j + 1 < degree:
                result[j] ^= result[j + 1]
        root = _gf_multiply(root, 0x02)
    return result


def _rs_remainder(data: list[int], divisor: list[int]) -> list[int]:
    result = [0] * len(divisor)
    for byte in data:
        factor = byte ^ result.pop(0)
        result.append(0)
        for i, coefficient in enumerate(divisor):
            result[i] ^= _gf_multiply(coefficient, factor)
    return result


# --- codewords ---------------------------------------------------------------


def _data_codewords(payload: bytes, version: int) -> list[int]:
    bits: list[int] = []

    def put(value: int, width: int) -> None:
        bits.extend((value >> i) & 1 for i in reversed(range(width)))

    capacity_bits = 8 * _data_capacity(version)
    put(0b0100, 4)  # byte mode
    put(len(payload), _count_bits(version))
    for byte in payload:
        put(byte, 8)
    put(0, min(4, capacity_bits - len(bits)))  # terminator
    put(0, -len(bits) % 8)
    codewords = [int("".join(map(str, bits[i:i + 8])), 2) for i in range(0, len(bits), 8)]
    pad = 0xEC
    while len(codewords) < _data_capacity(version):
        codewords.append(pad)
        pad ^= 0xEC ^ 0x11
    return codewords


def _interleaved(data: list[int], version: int) -> list[int]:
    ec_length, groups = _BLOCKS_M[version]
    divisor = _rs_divisor(ec_length)
    blocks: list[list[int]] = []
    offset = 0
    for count, size in groups:
        for _ in range(count):
            blocks.append(data[offset:offset + size])
            offset += size
    ec_blocks = [_rs_remainder(block, divisor) for block in blocks]
    out: list[int] = []
    for i in range(max(len(block) for block in blocks)):
        out.extend(block[i] for block in blocks if i < len(block))
    for i in range(ec_length):
        out.extend(block[i] for block in ec_blocks)
    return out


# --- the symbol --------------------------------------------------------------


class _Symbol:
    def __init__(self, version: int) -> None:
        self.version = version
        self.size = 4 * version + 17
        self.dark = [[False] * self.size for _ in range(self.size)]
        self.function = [[False] * self.size for _ in range(self.size)]

    def set_function(self, x: int, y: int, dark: bool) -> None:
        self.dark[y][x] = dark
        self.function[y][x] = True

    def draw_function_patterns(self) -> None:
        size = self.size
        for i in range(size):
            self.set_function(6, i, i % 2 == 0)
            self.set_function(i, 6, i % 2 == 0)
        for cx, cy in ((3, 3), (size - 4, 3), (3, size - 4)):
            for dy in range(-4, 5):
                for dx in range(-4, 5):
                    x, y = cx + dx, cy + dy
                    if 0 <= x < size and 0 <= y < size:
                        self.set_function(x, y, max(abs(dx), abs(dy)) not in (2, 4))
        positions = _ALIGNMENT[self.version]
        last = len(positions) - 1
        for i, cx in enumerate(positions):
            for j, cy in enumerate(positions):
                if (i, j) in ((0, 0), (0, last), (last, 0)):
                    continue  # those corners belong to the finder patterns
                for dy in range(-2, 3):
                    for dx in range(-2, 3):
                        self.set_function(cx + dx, cy + dy, max(abs(dx), abs(dy)) != 1)
        self.draw_format(0)  # reserves the format area; redrawn with the real mask
        self.draw_version()

    def draw_format(self, mask: int) -> None:
        data = (_FORMAT_BITS_M << 3) | mask
        remainder = data
        for _ in range(10):
            remainder = (remainder << 1) ^ ((remainder >> 9) * 0x537)
        bits = ((data << 10) | remainder) ^ 0x5412
        bit = lambda i: (bits >> i) & 1 == 1  # noqa: E731
        size = self.size
        for i in range(6):
            self.set_function(8, i, bit(i))
        self.set_function(8, 7, bit(6))
        self.set_function(8, 8, bit(7))
        self.set_function(7, 8, bit(8))
        for i in range(9, 15):
            self.set_function(14 - i, 8, bit(i))
        for i in range(8):
            self.set_function(size - 1 - i, 8, bit(i))
        for i in range(8, 15):
            self.set_function(8, size - 15 + i, bit(i))
        self.set_function(8, size - 8, True)  # the dark module

    def draw_version(self) -> None:
        if self.version < 7:
            return
        remainder = self.version
        for _ in range(12):
            remainder = (remainder << 1) ^ ((remainder >> 11) * 0x1F25)
        bits = (self.version << 12) | remainder
        for i in range(18):
            dark = (bits >> i) & 1 == 1
            a, b = self.size - 11 + i % 3, i // 3
            self.set_function(a, b, dark)
            self.set_function(b, a, dark)

    def draw_codewords(self, codewords: list[int]) -> None:
        size = self.size
        i = 0
        total = len(codewords) * 8
        right = size - 1
        while right >= 1:
            if right == 6:
                right = 5  # the vertical timing pattern is skipped whole
            upward = ((right + 1) & 2) == 0
            for vertical in range(size):
                y = size - 1 - vertical if upward else vertical
                for x in (right, right - 1):
                    if not self.function[y][x] and i < total:
                        self.dark[y][x] = (codewords[i >> 3] >> (7 - (i & 7))) & 1 == 1
                        i += 1
            right -= 2

    def apply_mask(self, mask: int) -> None:
        rule = _MASKS[mask]
        for y in range(self.size):
            for x in range(self.size):
                if not self.function[y][x] and rule(x, y):
                    self.dark[y][x] = not self.dark[y][x]


_MASKS = (
    lambda x, y: (x + y) % 2 == 0,
    lambda x, y: y % 2 == 0,
    lambda x, y: x % 3 == 0,
    lambda x, y: (x + y) % 3 == 0,
    lambda x, y: (x // 3 + y // 2) % 2 == 0,
    lambda x, y: x * y % 2 + x * y % 3 == 0,
    lambda x, y: (x * y % 2 + x * y % 3) % 2 == 0,
    lambda x, y: ((x + y) % 2 + x * y % 3) % 2 == 0,
)

_FINDER_LIKE = ("10111010000", "00001011101")


def _penalty(dark: list[list[bool]]) -> int:
    """ISO/IEC 18004 section 7.8.3's four rules, summed."""
    size = len(dark)
    lines = ["".join("1" if module else "0" for module in row) for row in dark]
    lines += ["".join("1" if dark[y][x] else "0" for y in range(size)) for x in range(size)]
    score = 0
    for line in lines:
        run_colour, run_length = None, 0
        for module in line + "x":  # the sentinel closes the last run
            if module == run_colour:
                run_length += 1
                continue
            if run_length >= 5:
                score += 3 + (run_length - 5)
            run_colour, run_length = module, 1
        for pattern in _FINDER_LIKE:
            start = line.find(pattern)
            while start != -1:
                score += 40
                start = line.find(pattern, start + 1)
    for y in range(size - 1):
        for x in range(size - 1):
            if dark[y][x] == dark[y][x + 1] == dark[y + 1][x] == dark[y + 1][x + 1]:
                score += 3
    dark_count = sum(sum(row) for row in dark)
    score += 10 * (abs(dark_count * 20 - size * size * 10) // (size * size))
    return score


def matrix(text: str) -> list[list[bool]]:
    """The QR code for text, without its quiet zone. True is dark."""
    if not text:
        raise ValueError("an empty QR code scans as nothing; refusing to draw one")
    payload = text.encode("utf-8")
    version = _pick_version(len(payload))
    codewords = _interleaved(_data_codewords(payload, version), version)

    best: tuple[int, list[list[bool]]] | None = None
    for mask in range(8):
        symbol = _Symbol(version)
        symbol.draw_function_patterns()
        symbol.draw_codewords(codewords)
        symbol.apply_mask(mask)
        symbol.draw_format(mask)
        score = _penalty(symbol.dark)
        if best is None or score < best[0]:
            best = (score, symbol.dark)
    assert best is not None
    return best[1]


def svg_path(modules: list[list[bool]], *, margin: int = QUIET_ZONE) -> str:
    """Path data for the dark modules, one horizontal run per subpath, offset
    by the quiet zone so the caller's viewBox is size + 2 * margin square."""
    parts: list[str] = []
    for y, row in enumerate(modules):
        x = 0
        while x < len(row):
            if not row[x]:
                x += 1
                continue
            start = x
            while x < len(row) and row[x]:
                x += 1
            parts.append(f"M{start + margin} {y + margin}h{x - start}v1h-{x - start}z")
    return "".join(parts)
