"""Where a donation to TBaguette goes: the Support dialog's Ko-fi page and
crypto addresses, read from donations.json at the repository root.

    python3 scripts/donations.py    # prints what the site would publish

donations.json is the single place a reviewer has to read to know where money
sent through the site ends up, so it holds public data only: the Ko-fi handle,
and one receive address per coin written by scripts/create_donation_wallets.py.
The seed behind those addresses never enters this repository. That script runs
on the maintainer's own machine, shows the seed once, and writes nothing here
but the addresses.

The coins are the twenty biggest by market cap, as CoinGecko ranked them on
2026-09-30, among the ones a wallet can be created for automatically: a coin
native to a public chain, whose receive address derives from one BIP-39 seed
with no account to register and no one else's permission. Tokens that live on
another chain (USDT, USDC, LINK, the other stablecoins, exchange tokens) are
not separate wallets -- they arrive at their host chain's address, which the
dialog says in one line. Also skipped from that ranking: Figure Heloc and
Canton, whose chains are permissioned, and Hedera, where an account exists
only once someone pays to create it.

This module is the build's half and is stdlib-only, like the rest of the
build. It checks every address against its coin's format before the site will
publish it, because a mistyped address burns whatever is sent to it and
nothing downstream can tell. It also refuses the addresses of the well-known
public test seeds: anyone can spend from those, and a fixture that leaked into
a commit would hand every donation to whoever sweeps them first.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path

DONATIONS_FILENAME = "donations.json"
KOFI_URL_TEMPLATE = "https://ko-fi.com/{handle}"
_KOFI_HANDLE = re.compile(r"[A-Za-z0-9_]{1,64}")

_BASE58 = "1-9A-HJ-NP-Za-km-z"
_BECH32 = "02-9ac-hj-np-z"
_EVM = r"0x[0-9a-fA-F]{40}"


@dataclass(frozen=True)
class Coin:
    """One entry in the dialog.

    symbol: the ticker, and the key in donations.json's "addresses".
    name: what the coin is called, per the same CoinGecko listing.
    network: the chain a donor must send on. For the coins that share the
        Ethereum address this is the whole difference between them.
    pattern: the full-match format of the receive address the generator
        writes -- not every valid address on that chain, only the kind this
        repository's wallets produce, so a pasted address of another kind is
        refused rather than trusted.
    note: one line a donor needs before sending, or "".
    """

    symbol: str
    name: str
    network: str
    pattern: str
    note: str = ""


# Market-cap order, as ranked on the date in the module docstring.
COINS: tuple[Coin, ...] = (
    Coin("BTC", "Bitcoin", "Bitcoin", rf"bc1q[{_BECH32}]{{38}}"),
    Coin("ETH", "Ethereum", "Ethereum", _EVM),
    Coin("BNB", "BNB", "BNB Smart Chain", _EVM),
    Coin("XRP", "XRP", "XRP Ledger", rf"r[{_BASE58}]{{24,34}}",
         "The first gift needs to be at least 1 XRP, the ledger's minimum to open an account."),
    Coin("SOL", "Solana", "Solana", rf"[{_BASE58}]{{32,44}}"),
    Coin("TRX", "TRON", "TRON", rf"T[{_BASE58}]{{33}}"),
    Coin("ZEC", "Zcash", "Zcash", rf"t1[{_BASE58}]{{33}}",
         "A transparent address: the gift is public on the chain, as with Bitcoin."),
    Coin("HYPE", "Hyperliquid", "Hyperliquid or HyperEVM", _EVM),
    Coin("DOGE", "Dogecoin", "Dogecoin", rf"D[{_BASE58}]{{33}}"),
    Coin("XMR", "Monero", "Monero", rf"4[{_BASE58}]{{94}}"),
    Coin("ADA", "Cardano", "Cardano", rf"addr1[{_BECH32}]{{98}}"),
    Coin("XLM", "Stellar", "Stellar", r"G[A-Z2-7]{55}",
         "The first gift needs to be at least 1 XLM, the network's minimum to open an account."),
    Coin("NEAR", "NEAR", "NEAR", r"[0-9a-f]{64}"),
    Coin("BCH", "Bitcoin Cash", "Bitcoin Cash", rf"bitcoincash:q[{_BECH32}]{{41}}"),
    Coin("LTC", "Litecoin", "Litecoin", rf"ltc1q[{_BECH32}]{{38}}"),
    Coin("AVAX", "Avalanche", "Avalanche C-Chain", _EVM),
    Coin("SUI", "Sui", "Sui", r"0x[0-9a-f]{64}"),
    Coin("GRAM", "Gram", "TON", r"UQ[A-Za-z0-9_-]{46}", "Formerly Toncoin."),
    Coin("TAO", "Bittensor", "Bittensor", rf"5[{_BASE58}]{{47}}"),
    Coin("CRO", "Cronos", "Cronos EVM", _EVM),
)

COINS_BY_SYMBOL = {coin.symbol: coin for coin in COINS}

# Tokens the dialog names in one line rather than as coins of their own: they
# arrive at these coins' addresses, on these coins' networks.
STABLECOIN_HOSTS = ("ETH", "TRX", "SOL")

# Receive addresses of the two BIP-39 test seeds every wallet library's test
# suite uses ("abandon" x11 + "about", and "abandon" x23 + "art"), derived
# exactly as scripts/create_donation_wallets.py derives the real ones. Their
# private keys are public; funds sent to them are swept by bots within
# minutes. scripts/test_donations.py re-derives these when bip_utils is
# installed, so this list cannot drift from the generator.
PUBLIC_TEST_ADDRESSES = frozenset({
    # abandon x11 about
    "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
    "0x9858EfFD232B4033E47d90003D41EC34EcaEda94",
    "rHsMGQEkVNJmpGWs8XUBoTBiAAbwxZN5v3",
    "GjJyeC1r2RgkuoCWMyPYkCWSGSGLcz266EaAkLA27AhL",
    "TUEZSdKsoDHQMeZwihtdoBiN46zxhGWYdH",
    "t1XVXWCvpMgBvUaed4XDqWtgQgJSu1Ghz7F",
    "DBus3bamQjgJULBJtYXpEzDWQRwF5iwxgC",
    "49vDbkSo7eve3J41sBdjvjaBUyz8qHohsQcGtRf63qEUTMBvmA45fpp5pSacMdSg7A3b71RejLzB8EkGbfjp5PELVF2N4Zn",
    "addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv",
    "GB3JDWCQJCWMJ3IILWIGDTQJJC5567PGVEVXSCVPEQOTDN64VJBDQBYX",
    "5510e2b44cae6eb807e3e0e45d579dda058c274abcba15e5cb84636f5d1ee412",
    "bitcoincash:qqyx49mu0kkn9ftfj6hje6g2wfer34yfnq5tahq3q6",
    "ltc1qjmxnz78nmc8nq77wuxh25n2es7rzm5c2rkk4wh",
    "0x5e93a736d04fbb25737aa40bee40171ef79f65fae833749e3c089fe7cc2161f1",
    "UQAzWZa6nM5mJev91wGc7VCSfBoIsYRqKJpV78N8Add9-RKY",
    "5EPCUjPxiHAcNooYipQFWr9NmmXJKpNG5RhcntXwbtUySrgH",
    # abandon x23 art (its TAO address is the one above: HMAC pads a short
    # all-zero key to the same block, so both seeds' zero entropy agrees)
    "bc1qzmtrqsfuaf6l6kkcsseumq26ukaphfj9skkug6",
    "0xF278cF59F82eDcf871d630F28EcC8056f25C1cdb",
    "rKxpJQ6hLWYbo7p1oo7WHjrcrRFv1TUQeC",
    "4BZp4ci5rhNYqbayj1uppeTas1osK2Q4b74x7UENC5Hd",
    "TEfhiqsW1SdN44DeHrAWVmbyr8ZbvChrtS",
    "t1dUDJ62ANtmebE8drFg7g2MWYwXHQ6Xu3F",
    "DL1DoPj4HvpnRT9n3YfCkhHXe5287wMyWD",
    "4AoBztrjij3U7dKo8AaQLLPG3haszmAH1TXwLbebyZdbUNUHs7Ly7ByG9PkxxPpZZEe1bhTBcJfze6qnBRvdjKG19KhWXfy",
    "addr1qyqt0pru382hy9vjlsxv3ye02z50sfvt8xunscg5pgden77z73dpdfng2ctw2ekqplqgrljelz7h4dneac27nn3qx3rqrhqvwd",
    "GB3TCCIC6KLYKM72PX7KA6RNYC2BHC7DQYDMEAAN7PDMUZQD7UKJGSSY",
    "5cd11aa446d8db56ace1bd3781673e351af6481b86c4660a8acd424973f9827b",
    "bitcoincash:qzlpu4vqftmmufrt2zvh8v2l8kskjuwllcmx6mydzm",
    "ltc1qj0xmcw3ttxgsfhzzcft9ac9nwp8smzq778lu3c",
    "0xf967e21c16a4757daafec13ee79c0dc5c5329199be5d70c86fd07b8e75db892c",
    "UQDOTGWodYYyrTkfGPECR7I9_B1-3BbF6Wzvgp0FIomdctqZ",
})


@dataclass(frozen=True)
class Support:
    """What the Support dialog shows. kofi_url is None when no handle is
    set; addresses is in COINS order and holds only the coins that have one."""

    kofi_url: str | None
    addresses: tuple[tuple[Coin, str], ...]

    @property
    def stablecoin_hosts(self) -> tuple[Coin, ...]:
        present = {coin.symbol for coin, _ in self.addresses}
        return tuple(COINS_BY_SYMBOL[s] for s in STABLECOIN_HOSTS if s in present)


class DonationsError(ValueError):
    """donations.json says something the site must not publish."""


def check_address(symbol: str, address: str) -> None:
    """Raises DonationsError unless address is a well-formed receive address
    for symbol and is not one of the public test addresses."""
    coin = COINS_BY_SYMBOL.get(symbol)
    if coin is None:
        known = ", ".join(COINS_BY_SYMBOL)
        raise DonationsError(f"{symbol!r} is not one of the Support dialog's coins ({known})")
    if not isinstance(address, str) or not re.fullmatch(coin.pattern, address):
        raise DonationsError(
            f"the {symbol} address {address!r} is not a {coin.network} receive address "
            "of the kind scripts/create_donation_wallets.py writes; anything sent to a "
            "mistyped address is lost, so the site refuses to publish it"
        )
    if address in PUBLIC_TEST_ADDRESSES:
        raise DonationsError(
            f"the {symbol} address {address} belongs to a public BIP-39 test seed, whose "
            "keys anyone can use; run scripts/create_donation_wallets.py for real ones"
        )


def parse(data: object) -> Support:
    if not isinstance(data, dict):
        raise DonationsError("donations.json must hold a JSON object")
    unknown_keys = set(data) - {"kofi", "created", "addresses"}
    if unknown_keys:
        raise DonationsError(f"donations.json has unexpected keys: {sorted(unknown_keys)}")

    handle = data.get("kofi")
    if handle is None or handle == "":
        kofi_url = None
    elif isinstance(handle, str) and _KOFI_HANDLE.fullmatch(handle):
        kofi_url = KOFI_URL_TEMPLATE.format(handle=handle)
    else:
        raise DonationsError(f"{handle!r} is not a Ko-fi handle (letters, digits, underscores)")

    addresses = data.get("addresses") or {}
    if not isinstance(addresses, dict):
        raise DonationsError('"addresses" must map coin symbols to addresses')
    for symbol, address in addresses.items():
        check_address(symbol, address)
    ordered = tuple((coin, addresses[coin.symbol]) for coin in COINS if coin.symbol in addresses)
    return Support(kofi_url=kofi_url, addresses=ordered)


def load(path: Path) -> Support | None:
    """The Support dialog's content, or None when path does not exist (a
    scratch project root in a test, or a fork that removed the file)."""
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise DonationsError(f"{path} is not valid JSON: {error}") from error
    return parse(data)


def main() -> None:
    path = Path(__file__).resolve().parent.parent / DONATIONS_FILENAME
    support = load(path)
    if support is None:
        print(f"no {DONATIONS_FILENAME}: the site builds without a Support button")
        return
    print(f"Ko-fi: {support.kofi_url or '(none)'}")
    if not support.addresses:
        print("crypto: none yet -- run scripts/create_donation_wallets.py")
    for coin, address in support.addresses:
        print(f"{coin.symbol:>5}  {coin.network:<24} {address}")


if __name__ == "__main__":
    main()
