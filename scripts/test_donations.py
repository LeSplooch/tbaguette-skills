"""Where a donation goes: donations.py's refusals, the committed
donations.json, the header's Support markup and the QR sprite behind it, and
-- when bip_utils is installed -- create_donation_wallets.py's derivations
against vectors published by the chains and wallets themselves.

The build is stdlib-only and so is most of this file. The derivation tests
need bip_utils, which only the wallet script uses, and skip without it; the
maintainer who runs that script has it installed, and that is the machine
where a derivation change would be made.

    python3 -m unittest test_donations -v
"""

from __future__ import annotations

import json
import re
import tempfile
import unittest
from pathlib import Path

import donations
import generate
import templates

REPO_ROOT = Path(__file__).resolve().parent.parent

# Pattern-valid and owned by nobody: every character is filler, so no checksum
# holds and no key produces them. Fixtures that look like addresses must never
# be addresses someone could hold the keys to.
FAKE = {
    "BTC": "bc1q" + "q" * 38,
    "ETH": "0x" + "1" * 40,
    "SOL": "1" * 40,
    "TRX": "T" + "1" * 33,
    "XMR": "4" + "1" * 94,
}

try:
    import bip_utils  # noqa: F401
    HAVE_BIP_UTILS = True
except ImportError:
    HAVE_BIP_UTILS = False


class Catalog(unittest.TestCase):
    def test_twenty_coins_with_distinct_symbols(self):
        self.assertEqual(len(donations.COINS), 20)
        self.assertEqual(len(donations.COINS_BY_SYMBOL), 20)

    def test_every_public_test_address_fits_some_coin(self):
        # A guard entry that fits no pattern could never fire, which would
        # mean it was mistyped when the list was written.
        for address in donations.PUBLIC_TEST_ADDRESSES:
            with self.subTest(address=address[:16]):
                self.assertTrue(any(re.fullmatch(c.pattern, address) for c in donations.COINS))

    def test_stablecoin_hosts_are_coins(self):
        for symbol in donations.STABLECOIN_HOSTS:
            self.assertIn(symbol, donations.COINS_BY_SYMBOL)


class Parse(unittest.TestCase):
    def test_kofi_only(self):
        support = donations.parse({"kofi": "tbaguette", "addresses": {}})
        self.assertEqual(support.kofi_url, "https://ko-fi.com/tbaguette")
        self.assertEqual(support.addresses, ())

    def test_addresses_come_back_in_market_cap_order(self):
        support = donations.parse({"addresses": {"XMR": FAKE["XMR"], "BTC": FAKE["BTC"], "SOL": FAKE["SOL"]}})
        self.assertEqual([c.symbol for c, _ in support.addresses], ["BTC", "SOL", "XMR"])
        self.assertIsNone(support.kofi_url)

    def test_refuses_an_address_of_the_wrong_chain(self):
        with self.assertRaisesRegex(donations.DonationsError, "not a Bitcoin receive address"):
            donations.parse({"addresses": {"BTC": FAKE["ETH"]}})

    def test_refuses_a_truncated_address(self):
        with self.assertRaises(donations.DonationsError):
            donations.parse({"addresses": {"ETH": FAKE["ETH"][:-1]}})

    def test_refuses_an_unknown_coin(self):
        with self.assertRaisesRegex(donations.DonationsError, "not one of"):
            donations.parse({"addresses": {"USDT": FAKE["ETH"]}})

    def test_refuses_public_test_seed_addresses(self):
        with self.assertRaisesRegex(donations.DonationsError, "public BIP-39 test seed"):
            donations.parse({"addresses": {"BTC": "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"}})

    def test_refuses_a_kofi_url_where_a_handle_belongs(self):
        with self.assertRaises(donations.DonationsError):
            donations.parse({"kofi": "https://ko-fi.com/someone-else"})

    def test_refuses_unexpected_keys(self):
        # A "seed" or "mnemonic" key in this file is the mistake this is for.
        with self.assertRaisesRegex(donations.DonationsError, "unexpected keys"):
            donations.parse({"kofi": "tbaguette", "mnemonic": "never"})


class CommittedFile(unittest.TestCase):
    def test_the_repositorys_donations_json_is_one_the_site_publishes(self):
        support = donations.load(REPO_ROOT / donations.DONATIONS_FILENAME)
        self.assertIsNotNone(support)
        self.assertIsNotNone(support.kofi_url)

    def test_holds_nothing_that_looks_like_a_seed(self):
        text = (REPO_ROOT / donations.DONATIONS_FILENAME).read_text(encoding="utf-8")
        # Twelve or more lowercase words in a row is a mnemonic's shape.
        self.assertIsNone(re.search(r"(?:\b[a-z]{3,8}\b[ ,]+){11,}\b[a-z]{3,8}\b", text))


class Header(unittest.TestCase):
    def tearDown(self):
        templates.SUPPORT = None
        templates.SUPPORT_QR_VERSION = ""

    def test_nothing_without_a_file(self):
        templates.SUPPORT = None
        self.assertEqual(templates._render_support(""), "")
        self.assertNotIn("data-support", templates._render_header("", ""))

    def test_kofi_only_has_no_crypto_section(self):
        templates.SUPPORT = donations.parse({"kofi": "tbaguette"})
        html = templates._render_support("/base")
        self.assertIn('href="https://ko-fi.com/tbaguette"', html)
        self.assertIn('rel="noopener"', html)
        self.assertNotIn("support__crypto", html)

    def test_every_address_is_on_the_page_with_its_network_and_copy_button(self):
        templates.SUPPORT = donations.parse({"kofi": "tbaguette", "addresses": FAKE})
        templates.SUPPORT_QR_VERSION = "abc123"
        html = templates._render_support("/base")
        for symbol, address in FAKE.items():
            coin = donations.COINS_BY_SYMBOL[symbol]
            slug = symbol.lower()
            with self.subTest(symbol=symbol):
                self.assertIn(f'id="support-address-{slug}">{address}</code>', html)
                self.assertIn(f'data-copy-target="support-address-{slug}"', html)
                self.assertIn(f"Send on <strong>{coin.network}</strong>", html)
                self.assertIn(f'/base/support/qr.svg?v=abc123#qr-{slug}"', html)
        self.assertIn("USDT and USDC are welcome too, at the Ethereum, TRON or Solana address", html)

    def test_seated_after_the_actions_in_source_order(self):
        templates.SUPPORT = donations.parse({"kofi": "tbaguette"})
        header = templates._render_header("", "")
        self.assertLess(header.index("data-theme-toggle"), header.index("site-header__support"))

    def test_sprite_has_one_symbol_per_address(self):
        support = donations.parse({"addresses": FAKE})
        sprite = templates.render_support_qr_sprite(support)
        ids = re.findall(r'<symbol id="qr-([a-z]+)"', sprite)
        self.assertEqual(ids, [c.symbol.lower() for c, _ in support.addresses])


class Build(unittest.TestCase):
    def test_a_refused_file_stops_the_build_before_anything_is_written(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / donations.DONATIONS_FILENAME).write_text(
                json.dumps({"addresses": {"BTC": FAKE["ETH"]}}), encoding="utf-8")
            with self.assertRaises(SystemExit) as caught:
                generate.generate(root, REPO_ROOT / "skills")
            self.assertIn("not a Bitcoin receive address", str(caught.exception))
            self.assertFalse((root / "docs" / "index.html").exists())
        templates.SUPPORT = None
        templates.SUPPORT_QR_VERSION = ""


@unittest.skipUnless(HAVE_BIP_UTILS, "bip_utils is not installed (only the wallet script needs it)")
class Derivations(unittest.TestCase):
    """Each vector below comes from the chain's or a wallet's own published
    tests, not from this script's output; together they pin both the maths
    and the path each wallet expects."""

    @staticmethod
    def derive_unguarded(mnemonic: str) -> tuple[dict[str, str], str]:
        import create_donation_wallets
        original = donations.check_address
        donations.check_address = lambda symbol, address: None
        try:
            return create_donation_wallets.derive(mnemonic)
        finally:
            donations.check_address = original

    ABOUT = "abandon " * 11 + "about"

    def test_published_vectors(self):
        addresses, _ = self.derive_unguarded(self.ABOUT)
        # BIP-84's own test vector.
        self.assertEqual(addresses["BTC"], "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu")
        # The address MetaMask and every EVM wallet show for this seed.
        self.assertEqual(addresses["ETH"], "0x9858EfFD232B4033E47d90003D41EC34EcaEda94")
        for symbol in ("BNB", "HYPE", "AVAX", "CRO"):
            self.assertEqual(addresses[symbol], addresses["ETH"])

    def test_stellar_sep_0005_vector(self):
        import create_donation_wallets
        bu = create_donation_wallets._bip_utils()
        mnemonic = "illness spike retreat truth genius clock brain pass fit cave bargain toe"
        seed = bu.Bip39SeedGenerator(mnemonic).Generate()
        account = bu.Bip44.FromSeed(seed, bu.Bip44Coins.STELLAR).Purpose().Coin().Account(0)
        self.assertEqual(account.PublicKey().ToAddress(),
                         "GDRXE2BQUC3AZNPVFSCEZ76NJ3WWL25FYFK6RGZGIEKWE4SOOHSUJUJ6")

    def test_substrate_dev_phrase_root(self):
        # polkadot.js: the root sr25519 account of the development phrase.
        import create_donation_wallets
        bu = create_donation_wallets._bip_utils()
        dev = "bottom drive obey lake curtain smoke basket hold race lonely fit walk"
        key = bu.Substrate.FromSeed(bu.SubstrateBip39SeedGenerator(dev).Generate(), bu.SubstrateCoins.GENERIC)
        self.assertEqual(key.PublicKey().ToAddress(), "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV")

    def test_cardano_payment_key_matches_cip_19(self):
        # CIP-19's test vectors use this mnemonic's first payment key.
        import create_donation_wallets
        bu = create_donation_wallets._bip_utils()
        mnemonic = "test walk nut penalty hip pave soap entry language right filter choice"
        account = bu.Cip1852.FromSeed(bu.CardanoIcarusSeedGenerator(mnemonic).Generate(),
                                      bu.Cip1852Coins.CARDANO_ICARUS).Purpose().Coin().Account(0)
        key = bu.CardanoShelley.FromCip1852Object(account).Change(bu.Bip44Changes.CHAIN_EXT) \
            .AddressIndex(0).PublicKeys().AddressKey().RawCompressed().ToBytes()[1:]
        self.assertEqual(bu.Bech32Encoder.Encode("addr_vk", key),
                         "addr_vk1w0l2sr2zgfm26ztc6nl9xy8ghsk5sh6ldwemlpmp9xylzy4dtf7st80zhd")

    def test_the_guard_list_is_exactly_what_the_test_seeds_derive(self):
        derived = set()
        for mnemonic in (self.ABOUT, "abandon " * 23 + "art"):
            addresses, _ = self.derive_unguarded(mnemonic)
            derived |= set(addresses.values())
        self.assertEqual(derived, donations.PUBLIC_TEST_ADDRESSES)

    def test_derive_refuses_a_public_test_seed(self):
        import create_donation_wallets
        with self.assertRaises(donations.DonationsError):
            create_donation_wallets.derive(self.ABOUT)

    def test_monero_seed_restores_the_same_address(self):
        import create_donation_wallets
        bu = create_donation_wallets._bip_utils()
        addresses, words = self.derive_unguarded(self.ABOUT)
        self.assertEqual(len(words.split()), 25)
        # What a Monero wallet does with the 25 words: decode them to the seed
        # and derive the keys from that, with nothing from the BIP-39 side.
        restored = bu.Monero.FromSeed(bu.MoneroSeedGenerator(words).Generate())
        self.assertEqual(str(restored.PrimaryAddress()), addresses["XMR"])

    def test_a_fresh_seed_derives_twenty_addresses_the_site_accepts(self):
        import create_donation_wallets
        mnemonic = create_donation_wallets.new_mnemonic()
        self.assertEqual(len(mnemonic.split()), 24)
        self.assertTrue(create_donation_wallets.is_valid_mnemonic(mnemonic))
        addresses, _ = create_donation_wallets.derive(mnemonic)
        support = donations.parse({"addresses": addresses})
        self.assertEqual(len(support.addresses), 20)


if __name__ == "__main__":
    unittest.main()
