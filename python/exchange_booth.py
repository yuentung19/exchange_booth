import argparse
from typing import NamedTuple
import struct
import base64
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.rpc.api import Client
from solana.rpc.types import TxOpts
from solana.rpc.commitment import Confirmed
from solana.system_program import CreateAccountParams, create_account, SYS_PROGRAM_ID
from solana.transaction import AccountMeta, TransactionInstruction, Transaction

from spl.token.client import Token
from spl.token.constants import TOKEN_PROGRAM_ID

pack_str = lambda s: struct.pack("<I" + (len(s) * "B"), len(s), *s.encode("ascii"))


class InitExchangeBoothParams(NamedTuple):
    program_id: PublicKey
    admin: PublicKey  # [S]
    exchange_booth: PublicKey  # [W]
    oracle: PublicKey
    vault_a: PublicKey
    vault_b: PublicKey


def init_exchange_booth(params: InitExchangeBoothParams) -> TransactionInstruction:
    # no data needed for init
    data = b""

    return TransactionInstruction(
        keys=[
            AccountMeta(pubkey=params.admin, is_signer=True, is_writable=False),
            AccountMeta(
                pubkey=params.exchange_booth, is_signer=False, is_writable=True
            ),
            AccountMeta(pubkey=params.oracle, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.vault_a, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.vault_b, is_signer=False, is_writable=True),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        program_id=params.program_id,
        data=data,
    )


def main_init(args, client):
    program_id = PublicKey(args.program_id)
    fee_payer = Keypair()

    print("Requesting Airdrop of 2 SOL...")
    client.request_airdrop(fee_payer.public_key, int(2e9))
    print("Airdrop received")

    admin = Keypair()
    oracle = Keypair()
    exchange_booth = Keypair()

    token_a = Token.create_mint(
        client,
        fee_payer,
        fee_payer.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint_a = token_a.pubkey

    token_b = Token.create_mint(
        client,
        fee_payer,
        fee_payer.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint_b = token_b.pubkey

    # create PDA for 'vault_a'
    vault_a, _ = PublicKey.find_program_address(
        [
            b"vault_a",
            bytes(admin.public_key),
            bytes(exchange_booth.public_key),
            bytes(mint_a),
        ],
        program_id,
    )

    # create PDA for 'vault_b'
    vault_b, _ = PublicKey.find_program_address(
        [
            b"vault_b",
            bytes(admin.public_key),
            bytes(exchange_booth.public_key),
            bytes(mint_b),
        ],
        program_id,
    )

    init_exchange_booth_ix = init_exchange_booth(
        InitExchangeBoothParams(
            program_id=program_id,
            admin=admin.public_key,
            exchange_booth=exchange_booth.public_key,
            oracle=oracle.public_key,
            vault_a=vault_a,
            vault_b=vault_b,
        )
    )

    tx = Transaction().add(init_exchange_booth_ix)
    signers = [admin]

    result = client.send_transaction(
        tx,
        *signers,
        opts=TxOpts(
            skip_preflight=True,
        ),
    )

    return result


def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(help="types of instruction", dest="command")
    init_parser = subparsers.add_parser("init")

    # arguments for init
    init_parser.add_argument(
        "program_id",
        help="Devnet program ID (base58 encoded string) of the deployed Echo Program",
    )

    args = parser.parse_args()
    client = Client("https://api.devnet.solana.com")

    if args.command == "init":
        result = main_init(args, client)
    else:
        raise RuntimeError(f"{args.command} not supported yet")

    tx_hash = result["result"]
    client.confirm_transaction(tx_hash, commitment="confirmed")
    print(f"https://explorer.solana.com/tx/{tx_hash}?cluster=devnet")


if __name__ == "__main__":
    main()
