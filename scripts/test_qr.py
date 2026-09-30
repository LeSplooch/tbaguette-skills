"""qr.py draws the codes a donor's phone reads an address from, so a wrong
module is a gift sent nowhere. What is checked here:

* Reed-Solomon against the worked example every QR tutorial uses (HELLO
  WORLD, 1-M), a vector this module did not produce;
* the symbol's own bookkeeping read back out of it -- finder patterns, the
  format bits' BCH code, the level, the version -- rather than trusted;
* two whole symbols against fingerprints recorded when they were compared,
  bit for bit, with Project Nayuki's reference encoder at the same mask, and
  decoded by zxing-cpp, an independent decoder. Neither tool is needed to
  run this file; the fingerprints stand in for them until the encoder
  changes, and a change that moves one should be re-verified the same way.

    python3 -m unittest test_qr -v
"""

from __future__ import annotations

import hashlib
import unittest

import qr

BTC = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
ADA = ("addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3"
       "c7revpf7jx0xnlcjz3g69mq4afdhv")


def fingerprint(modules: list[list[bool]]) -> str:
    bits = "".join("1" if dark else "0" for row in modules for dark in row)
    return hashlib.sha256(bits.encode()).hexdigest()


def format_word(modules: list[list[bool]]) -> int:
    """The 15 format bits from the copy around the top-left finder."""
    positions = ([(8, i) for i in range(6)] + [(8, 7), (8, 8), (7, 8)]
                 + [(14 - i, 8) for i in range(9, 15)])
    word = 0
    for i, (x, y) in enumerate(positions):
        word |= int(modules[y][x]) << i
    return word


def bch_remainder(value: int, generator: int, degree: int) -> int:
    top = generator.bit_length() - 1
    value <<= degree
    for shift in reversed(range(value.bit_length() - top)):
        if value >> (shift + top) & 1:
            value ^= generator << shift
    return value


class ReedSolomon(unittest.TestCase):
    def test_hello_world_1m_worked_example(self):
        data = [32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 17, 236, 17]
        self.assertEqual(qr._rs_remainder(data, qr._rs_divisor(10)),
                         [196, 35, 39, 119, 235, 215, 231, 226, 93, 23])


class Capacity(unittest.TestCase):
    def test_versions_at_their_byte_mode_limits(self):
        # ISO/IEC 18004 table 7, byte mode, level M.
        for length, version in ((14, 1), (15, 2), (106, 6), (107, 7), (213, 10)):
            with self.subTest(length=length):
                self.assertEqual(len(qr.matrix("x" * length)), 4 * version + 17)

    def test_refuses_what_it_cannot_hold_rather_than_truncating(self):
        with self.assertRaises(ValueError):
            qr.matrix("x" * 214)

    def test_refuses_an_empty_code(self):
        with self.assertRaises(ValueError):
            qr.matrix("")

    def test_the_longest_address_the_dialog_carries_fits(self):
        self.assertEqual(len(qr.matrix(ADA)), 4 * 6 + 17)


class Structure(unittest.TestCase):
    def test_finder_patterns_in_three_corners(self):
        modules = qr.matrix(BTC)
        size = len(modules)
        expected = [[max(abs(dx - 3), abs(dy - 3)) != 2 for dx in range(7)] for dy in range(7)]
        for x0, y0 in ((0, 0), (size - 7, 0), (0, size - 7)):
            with self.subTest(corner=(x0, y0)):
                got = [[modules[y0 + dy][x0 + dx] for dx in range(7)] for dy in range(7)]
                self.assertEqual(got, expected)

    def test_format_bits_are_a_valid_code_word_for_level_m(self):
        for text in (BTC, ADA, "a"):
            with self.subTest(text=text[:12]):
                word = format_word(qr.matrix(text)) ^ 0x5412
                data = word >> 10
                self.assertEqual(bch_remainder(data, 0x537, 10), word & 0x3FF)
                self.assertEqual(data >> 3, 0b00, "the level bits say M")

    def test_both_format_copies_agree(self):
        modules = qr.matrix(ADA)
        size = len(modules)
        second = 0
        for i in range(8):
            second |= int(modules[8][size - 1 - i]) << i
        for i in range(8, 15):
            second |= int(modules[size - 15 + i][8]) << i
        self.assertEqual(second, format_word(modules))

    def test_dark_module(self):
        modules = qr.matrix(BTC)
        self.assertTrue(modules[len(modules) - 8][8])


class Golden(unittest.TestCase):
    """Recorded 2026-09-30 after both symbols matched qrcodegen 1.8 at the
    same mask (2, for both) and decoded under zxing-cpp."""

    def test_bitcoin_address(self):
        self.assertEqual(fingerprint(qr.matrix(BTC)),
                         "4b23ae5a3abc02f214bbcdb5441c6041ff0c6dea9c97eab6e33cabb1fa1f665a")

    def test_cardano_address(self):
        self.assertEqual(fingerprint(qr.matrix(ADA)),
                         "2652b624162e570a9cfa532c4058a7a3282260f8bc13506780d70e7c6246bc7c")


class SvgPath(unittest.TestCase):
    def test_paints_exactly_the_dark_modules_inside_the_quiet_zone(self):
        modules = qr.matrix(BTC)
        size = len(modules)
        painted = [[False] * size for _ in range(size)]
        for run in qr.svg_path(modules).split("z")[:-1]:
            x, rest = run[1:].split(" ")
            y, width = rest.split("h", 1)
            width = int(width.split("v")[0])
            x, y = int(x) - qr.QUIET_ZONE, int(y) - qr.QUIET_ZONE
            for dx in range(width):
                self.assertFalse(painted[y][x + dx], "no module painted twice")
                painted[y][x + dx] = True
        self.assertEqual(painted, modules)


if __name__ == "__main__":
    unittest.main()
