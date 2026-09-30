"""Creates the crypto wallets behind the site's Support button, on your own
machine, and writes their public addresses to donations.json.

    python3 -m venv ~/.venvs/tbaguette-wallets
    ~/.venvs/tbaguette-wallets/bin/pip install bip_utils
    ~/.venvs/tbaguette-wallets/bin/python scripts/create_donation_wallets.py

    ... --check      type the 24 words back in: do they match donations.json?
    ... --restore    rewrite donations.json from 24 words you already hold

One BIP-39 seed of 24 words controls all twenty wallets. The script shows it
once and saves nothing until you have typed it back from paper, so a
donations.json that exists is proof a backup of its seed exists too. The seed
never touches the disk, this repository, or the network: the only file written
is donations.json, and it holds addresses only.

Every derivation follows a mainstream wallet's defaults, so the same 24 words
restore the funds without this script. The paths and wallets are in
RESTORE_GUIDE below and are printed after every run. Monero is the exception
that cannot follow a BIP-39 wallet, because Monero wallets do not read BIP-39
seeds: its keys are derived from the same 24 words and handed over as Monero's
own 25-word seed, which --restore prints again whenever you need it.

bip_utils is the one dependency, and this script is the only thing in the
repository that needs it: the site build reads the addresses and stays
stdlib-only. Run it in a virtualenv, as above, rather than installing crypto
libraries into the Python your system tools use.
"""

from __future__ import annotations

import argparse
import getpass
import json
import secrets
import sys
from datetime import datetime, timezone
from pathlib import Path

import donations

REPO_ROOT = Path(__file__).resolve().parent.parent
DONATIONS_PATH = REPO_ROOT / donations.DONATIONS_FILENAME
WORD_COUNT = 24

EVM_SYMBOLS = ("ETH", "BNB", "HYPE", "AVAX", "CRO")

# symbol -> derivation, as a wallet asks for it on import. Every path here is
# Trust Wallet's default for that chain (wallet-core's registry.json), which is
# why one wallet restores eighteen of the twenty with no settings changed.
PATHS = {
    "BTC": "m/84'/0'/0'/0/0 (native SegWit)",
    "ETH": "m/44'/60'/0'/0/0",
    "BNB": "m/44'/60'/0'/0/0",
    "XRP": "m/44'/144'/0'/0/0",
    "SOL": "m/44'/501'/0'",
    "TRX": "m/44'/195'/0'/0/0",
    "ZEC": "m/44'/133'/0'/0/0 (transparent)",
    "HYPE": "m/44'/60'/0'/0/0",
    "DOGE": "m/44'/3'/0'/0/0",
    "XMR": "Monero 25-word seed (spend key from m/44'/128'/0'/0/0)",
    "ADA": "m/1852'/1815'/0'/0/0, staking m/1852'/1815'/0'/2/0 (Icarus)",
    "XLM": "m/44'/148'/0'",
    "NEAR": "m/44'/397'/0' (implicit account)",
    "BCH": "m/44'/145'/0'/0/0",
    "LTC": "m/84'/2'/0'/0/0 (native SegWit)",
    "AVAX": "m/44'/60'/0'/0/0 (C-Chain)",
    "SUI": "m/44'/784'/0'/0'/0'",
    "GRAM": "m/44'/607'/0', wallet v4R2",
    "TAO": "sr25519, no derivation path",
    "CRO": "m/44'/60'/0'/0/0",
}

RESTORE_GUIDE = """\
Where the money is, when you want to move it

  Trust Wallet, restored from the 24 words, shows these with its defaults:
    BTC ETH BNB XRP SOL TRX ZEC DOGE ADA XLM NEAR BCH LTC AVAX SUI GRAM CRO,
    and HYPE sent on HyperEVM. HYPE sent on Hyperliquid itself shows up at
    app.hyperliquid.xyz with the same Ethereum account connected.
  Phantom, Solflare: SOL too (Phantom lists this path among the ones it finds).
  TAO: Talisman, Nova Wallet or `btcli wallet regen-coldkey`, from the 24
    words, as an sr25519 account with no derivation path.
  XMR: Cake Wallet, Feather or the Monero GUI, from the 25 Monero words,
    restoring from the date donations.json records as "created".

  For any other wallet, the paths:
"""


def _bip_utils():
    try:
        import bip_utils
    except ImportError:
        raise SystemExit(
            "this script needs bip_utils, and only this script does:\n"
            "  python3 -m venv ~/.venvs/tbaguette-wallets\n"
            "  ~/.venvs/tbaguette-wallets/bin/pip install bip_utils\n"
            "  ~/.venvs/tbaguette-wallets/bin/python scripts/create_donation_wallets.py"
        ) from None
    return bip_utils


def normalise(words: str) -> str:
    return " ".join(words.lower().split())


def is_valid_mnemonic(mnemonic: str) -> bool:
    bu = _bip_utils()
    return len(mnemonic.split()) in (12, 15, 18, 21, 24) and bu.Bip39MnemonicValidator().IsValid(mnemonic)


def new_mnemonic() -> str:
    """24 words from 256 bits of the operating system's CSPRNG."""
    bu = _bip_utils()
    return bu.Bip39MnemonicEncoder(bu.Bip39Languages.ENGLISH).Encode(secrets.token_bytes(32)).ToStr()


def derive(mnemonic: str) -> tuple[dict[str, str], str]:
    """Every coin's receive address, in donations.COINS order, and the Monero
    wallet's own 25-word seed."""
    bu = _bip_utils()
    seed = bu.Bip39SeedGenerator(mnemonic).Generate()
    ext = bu.Bip44Changes.CHAIN_EXT

    def bip44(coin):
        return bu.Bip44.FromSeed(seed, coin).Purpose().Coin().Account(0)

    def first(ctx):
        return ctx.Change(ext).AddressIndex(0).PublicKey().ToAddress()

    evm = first(bip44(bu.Bip44Coins.ETHEREUM))
    found = {symbol: evm for symbol in EVM_SYMBOLS}
    found["BTC"] = bu.Bip84.FromSeed(seed, bu.Bip84Coins.BITCOIN).Purpose().Coin().Account(0) \
        .Change(ext).AddressIndex(0).PublicKey().ToAddress()
    found["LTC"] = bu.Bip84.FromSeed(seed, bu.Bip84Coins.LITECOIN).Purpose().Coin().Account(0) \
        .Change(ext).AddressIndex(0).PublicKey().ToAddress()
    found["XRP"] = first(bip44(bu.Bip44Coins.RIPPLE))
    found["TRX"] = first(bip44(bu.Bip44Coins.TRON))
    found["ZEC"] = first(bip44(bu.Bip44Coins.ZCASH))
    found["DOGE"] = first(bip44(bu.Bip44Coins.DOGECOIN))
    found["BCH"] = first(bip44(bu.Bip44Coins.BITCOIN_CASH))
    found["SUI"] = first(bip44(bu.Bip44Coins.SUI))
    # ed25519 chains whose wallets stop at the account level.
    found["SOL"] = bip44(bu.Bip44Coins.SOLANA).PublicKey().ToAddress()
    found["XLM"] = bip44(bu.Bip44Coins.STELLAR).PublicKey().ToAddress()
    found["NEAR"] = bip44(bu.Bip44Coins.NEAR_PROTOCOL).PublicKey().ToAddress()
    # Non-bounceable (UQ...), which is what an address that has never sent a
    # transaction must be given as: a bounceable transfer to an undeployed
    # wallet returns to the sender.
    found["GRAM"] = bip44(bu.Bip44Coins.TON).PublicKey().ToAddress()

    cardano = bu.Cip1852.FromSeed(bu.CardanoIcarusSeedGenerator(mnemonic).Generate(),
                                  bu.Cip1852Coins.CARDANO_ICARUS).Purpose().Coin().Account(0)
    found["ADA"] = bu.CardanoShelley.FromCip1852Object(cardano).Change(ext).AddressIndex(0) \
        .PublicKeys().ToAddress()

    substrate = bu.Substrate.FromSeed(bu.SubstrateBip39SeedGenerator(mnemonic).Generate(),
                                      bu.SubstrateCoins.GENERIC)
    found["TAO"] = substrate.PublicKey().ToAddress()

    monero_key = bip44(bu.Bip44Coins.MONERO_SECP256K1).Change(ext).AddressIndex(0) \
        .PrivateKey().Raw().ToBytes()
    monero = bu.Monero.FromBip44PrivateKey(monero_key)
    found["XMR"] = str(monero.PrimaryAddress())
    monero_words = bu.MoneroMnemonicEncoder(bu.MoneroLanguages.ENGLISH) \
        .EncodeWithChecksum(monero.PrivateSpendKey().Raw().ToBytes()).ToStr()

    addresses = {coin.symbol: found[coin.symbol] for coin in donations.COINS}
    for symbol, address in addresses.items():
        donations.check_address(symbol, address)
    return addresses, monero_words


# --- donations.json ----------------------------------------------------------


def read_file() -> dict:
    if not DONATIONS_PATH.is_file():
        return {}
    data = json.loads(DONATIONS_PATH.read_text(encoding="utf-8"))
    donations.parse(data)  # refuse to build on a file the site would refuse
    return data


def write_file(existing: dict, addresses: dict[str, str], created: str) -> None:
    data = {"kofi": existing.get("kofi"), "created": created, "addresses": addresses}
    DONATIONS_PATH.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def refuse_to_replace(existing: dict, addresses: dict[str, str], replace: bool) -> None:
    current = existing.get("addresses") or {}
    if current and current != addresses and not replace:
        raise SystemExit(
            f"{DONATIONS_PATH.name} already lists addresses from another seed. Replacing "
            "them strands anything sent to the old ones unless you still hold that seed "
            "(check with --check). Pass --replace if that is what you mean."
        )


# --- terminal ------------------------------------------------------------------


def show_words(words: list[str]) -> None:
    rows = (len(words) + 3) // 4
    for row in range(rows):
        cells = [f"{i + 1:>2}. {words[i]:<10}" for i in range(row, len(words), rows)]
        print("   " + "  ".join(cells))


def clear_screen() -> None:
    # 3J clears the scrollback as well as the screen, in terminals that honour it.
    sys.stdout.write("\033[3J\033[H\033[2J")
    sys.stdout.flush()


def ask_words(prompt: str) -> str:
    return normalise(getpass.getpass(prompt))


def confirm_backup(mnemonic: str) -> None:
    expected = mnemonic.split()
    while True:
        typed = ask_words(f"Type the {len(expected)} words from your paper, separated by spaces (hidden): ").split()
        if typed == expected:
            return
        if len(typed) != len(expected):
            print(f"  That was {len(typed)} words, not {len(expected)}. Nothing is saved yet; try again.")
            continue
        wrong = [str(i + 1) for i, (a, b) in enumerate(zip(typed, expected)) if a != b]
        print(f"  Word{'s' if len(wrong) > 1 else ''} {', '.join(wrong)} "
              "did not match. Nothing is saved yet; try again, or Ctrl-C to start over.")


def print_result(addresses: dict[str, str], monero_words: str, created: str) -> None:
    print("\nAddresses written to donations.json:")
    for coin in donations.COINS:
        print(f"  {coin.symbol:>5}  {addresses[coin.symbol]}")
    print("\nMonero's own seed, for a Monero wallet (it comes from the 24 words; "
          "--restore shows it again):")
    show_words(monero_words.split())
    print(f"   restore date: {created}\n")
    print(RESTORE_GUIDE, end="")
    for coin in donations.COINS:
        print(f"    {coin.symbol:>5}  {PATHS[coin.symbol]}")
    print(
        "\nBefore you push: restore the 24 words in Trust Wallet and check that its\n"
        "Bitcoin, Solana and Cardano addresses match the ones above. Then:\n"
        "  python3 scripts/generate.py --base-path /tbaguette-skills\n"
        "  git add donations.json docs && git commit && git push\n"
        "Clear this terminal when you are done; the Monero words are still on it."
    )


# --- modes -------------------------------------------------------------------------


def create(replace: bool) -> None:
    if not (sys.stdin.isatty() and sys.stdout.isatty()):
        raise SystemExit("run this in an interactive terminal: it shows a seed that must not end up in a log")
    existing = read_file()
    if (existing.get("addresses") or {}) and not replace:
        raise SystemExit(
            f"{DONATIONS_PATH.name} already lists addresses. A new seed would strand "
            "anything sent to them unless you still hold the old one (check with --check). "
            "Pass --replace if that is what you mean, or --restore to rewrite the file "
            "from the seed you have."
        )
    mnemonic = new_mnemonic()
    addresses, monero_words = derive(mnemonic)

    print(f"\nYour donation wallets' seed: {WORD_COUNT} words that control every coin below.\n")
    show_words(mnemonic.split())
    print(
        "\nWrite them on paper, in order. Anyone who sees them can take every donation;\n"
        "without them, nobody -- you included -- can get a donation back out.\n"
        "Nothing is saved until you type them back."
    )
    input("\nPress Enter once they are written down. The screen will clear. ")
    clear_screen()
    confirm_backup(mnemonic)

    created = datetime.now(timezone.utc).date().isoformat()
    write_file(existing, addresses, created)
    print_result(addresses, monero_words, created)


def restore(replace: bool) -> None:
    mnemonic = ask_words("Your 24 words, separated by spaces (hidden): ")
    if not is_valid_mnemonic(mnemonic):
        raise SystemExit("those words are not a valid BIP-39 seed (a word is misspelt or out of order)")
    addresses, monero_words = derive(mnemonic)
    existing = read_file()
    refuse_to_replace(existing, addresses, replace)
    if (existing.get("addresses") or {}) == addresses and existing.get("created"):
        created = existing["created"]
        print(f"{DONATIONS_PATH.name} already holds these addresses; left unchanged.")
    else:
        created = datetime.now(timezone.utc).date().isoformat()
        write_file(existing, addresses, created)
        print("If this seed received Monero before today, restore the Monero wallet from an earlier date.")
    print_result(addresses, monero_words, created)


def check() -> int:
    existing = read_file()
    published = existing.get("addresses") or {}
    if not published:
        raise SystemExit(f"{DONATIONS_PATH.name} has no addresses to check yet")
    mnemonic = ask_words("Your 24 words, separated by spaces (hidden): ")
    if not is_valid_mnemonic(mnemonic):
        print("Those words are not a valid BIP-39 seed: a word is misspelt or out of order.")
        return 1
    addresses, _ = derive(mnemonic)
    mismatched = [s for s, a in published.items() if addresses.get(s) != a]
    for symbol, address in published.items():
        print(f"  {'ok' if symbol not in mismatched else 'NO':>2}  {symbol:>5}  {address}")
    if mismatched:
        print(f"\nThese words do not control {', '.join(mismatched)}. Keep looking for the right paper.")
        return 1
    print(f"\nThese words control every address in {DONATIONS_PATH.name}.")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true",
                      help="type the seed in and confirm it controls donations.json's addresses")
    mode.add_argument("--restore", action="store_true",
                      help="rewrite donations.json from a seed you already hold")
    parser.add_argument("--replace", action="store_true",
                        help="allow overwriting addresses that came from a different seed")
    args = parser.parse_args(argv)
    try:
        if args.check:
            return check()
        if args.restore:
            restore(args.replace)
        else:
            create(args.replace)
    except donations.DonationsError as error:
        # A public test seed typed into --check or --restore, or a
        # donations.json the site would refuse: a sentence, not a traceback.
        raise SystemExit(str(error)) from None
    except KeyboardInterrupt:
        raise SystemExit("\nStopped.") from None
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
