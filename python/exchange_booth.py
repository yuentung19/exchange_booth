import sys
import argparse
import logging
from typing import List, NamedTuple, Tuple, Optional
import struct
import base64
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.rpc.api import Client
from solana.rpc.types import TxOpts
from solana.rpc.commitment import Confirmed
from solana.system_program import CreateAccountParams, create_account, SYS_PROGRAM_ID
from solana.transaction import AccountMeta, TransactionInstruction, Transaction
from solana.sysvar import SYSVAR_RENT_PUBKEY

from spl.token.client import Token
from spl.token.constants import TOKEN_PROGRAM_ID

from utils import CommandParams

logger = logging.getLogger(__name__)

pack_str = lambda s: struct.pack("<I" + (len(s) * "B"), len(s), *s.encode("ascii"))


class InitExchangeBoothParams(NamedTuple):
    program_id: PublicKey
    admin: PublicKey  # [S]
    exchange_booth: PublicKey  # [W]
    oracle: PublicKey
    vault_a: PublicKey  # [W]
    mint_a: PublicKey
    vault_b: PublicKey  # [W]
    mint_b: PublicKey


class SetExchangeRateParams(NamedTuple):
    program_id: PublicKey
    oracle: PublicKey
    exchange_rate_a_to_b: float


def init_exchange_booth(params: InitExchangeBoothParams) -> TransactionInstruction:
    # no data needed for init
    data = struct.pack("<B", 0)

    return TransactionInstruction(
        keys=[
            AccountMeta(
                pubkey=params.exchange_booth, is_signer=False, is_writable=True
            ),
            AccountMeta(pubkey=params.oracle, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.vault_a, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.vault_b, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.mint_a, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.mint_b, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.admin, is_signer=True, is_writable=False),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=SYSVAR_RENT_PUBKEY, is_signer=False, is_writable=False),
        ],
        program_id=params.program_id,
        data=data,
    )


def set_exchange_rate(params: SetExchangeRateParams) -> TransactionInstruction:
    # combine with exchange rate
    data = b"".join(
        [struct.pack("<B", 5), struct.pack("<d", params.exchange_rate_a_to_b)]
    )

    return TransactionInstruction(
        keys=[AccountMeta(pubkey=params.oracle, is_signer=False, is_writable=True)],
        program_id=params.program_id,
        data=data,
    )


def init(program_id, client) -> CommandParams:
    program_id = PublicKey(program_id)
    admin = Keypair()

    print("Requesting Airdrop of 2 SOL...")
    client.request_airdrop(admin.public_key, int(2e9))
    print("Airdrop received")

    oracle = Keypair()
    exchange_booth = Keypair()

    ixs = []
    # create accounts and allocate space
    for _account, _space in [(oracle, 16), (exchange_booth, 32 * 4)]:
        ixs.append(
            create_account(
                CreateAccountParams(
                    from_pubkey=admin.public_key,
                    new_account_pubkey=_account.public_key,
                    lamports=client.get_minimum_balance_for_rent_exemption(2022)[
                        "result"
                    ],
                    space=_space,
                    program_id=program_id,
                )
            )
        )

    token_a = Token.create_mint(
        client,
        admin,
        admin.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint_a = token_a.pubkey

    token_b = Token.create_mint(
        client,
        admin,
        admin.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint_b = token_b.pubkey

    # create PDA for 'vault_a'
    vault_a, _ = PublicKey.find_program_address(
        [
            bytes(admin.public_key),
            bytes(exchange_booth.public_key),
            bytes(mint_a),
        ],
        program_id,
    )

    # create PDA for 'vault_b'
    vault_b, _ = PublicKey.find_program_address(
        [
            bytes(admin.public_key),
            bytes(exchange_booth.public_key),
            bytes(mint_b),
        ],
        program_id,
    )

    params = InitExchangeBoothParams(
        program_id=program_id,
        admin=admin.public_key,
        exchange_booth=exchange_booth.public_key,
        oracle=oracle.public_key,
        vault_a=vault_a,
        mint_a=mint_a,
        vault_b=vault_b,
        mint_b=mint_b,
    )

    ixs.append(init_exchange_booth(params))
    signers = [admin, oracle, exchange_booth]

    return CommandParams(instructions=ixs, signers=signers, params=params)


def set_rate(
    program_id, client, exchange_rate_a_to_b: float, oracle: Optional[PublicKey] = None
):
    program_id = PublicKey(program_id)
    ixs = []
    signers = []
    admin = Keypair()
    print("Requesting Airdrop of 2 SOL...")
    client.request_airdrop(admin.public_key, int(2e9))
    print("Airdrop received")

    if oracle is None:
        print("Creating oracle account because init didn't run")
        oracle = Keypair()
        ixs.append(
            create_account(
                CreateAccountParams(
                    from_pubkey=admin.public_key,
                    new_account_pubkey=oracle.public_key,
                    lamports=client.get_minimum_balance_for_rent_exemption(40)[
                        "result"
                    ],
                    space=16,
                    program_id=program_id,
                )
            )
        )
        signers.append(oracle)
        oracle = oracle.public_key

    params = SetExchangeRateParams(
        program_id=program_id,
        oracle=oracle,
        exchange_rate_a_to_b=exchange_rate_a_to_b,
    )

    ixs.append(set_exchange_rate(params))
    signers.append(admin)

    return CommandParams(instructions=ixs, signers=signers, params=params)


def main():
    client = Client("https://api.devnet.solana.com")
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "program_id",
        help="Devnet program ID (base58 encoded string) of the deployed Echo Program",
    )
    args = parser.parse_args()
    ixs_supported = ("init", "set_rate", "exit")

    command_params = {}

    while True:
        command_input = input(f"Enter command from {ixs_supported}:\n")
        if command_input == "init":
            _params = init(args.program_id, client)
            command_params["init"] = _params

        elif command_input == "set_rate":
            try:
                _oracle = command_params["init"].params.oracle
            except KeyError:
                print("ENTERING TESTING MODE of 'set_rate'")
                _oracle = None

            _rate = float(input(f"Enter 'exchange_rate_a_to_b':\n"))
            _params = set_rate(args.program_id, client, _rate, _oracle)
            command_params["set_rate"] = _params

        elif command_input == "exit":
            sys.exit(0)

        else:
            raise RuntimeError(f"{command_input} not supported yet")

        result = client.send_transaction(
            Transaction().add(*(ix for ix in _params.instructions)),
            *_params.signers,
            opts=TxOpts(
                skip_preflight=True,
            ),
        )
        tx_hash = result["result"]
        client.confirm_transaction(tx_hash, commitment="confirmed")
        print(f"https://explorer.solana.com/tx/{tx_hash}?cluster=devnet")


if __name__ == "__main__":
    main()
