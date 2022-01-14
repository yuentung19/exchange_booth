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

admin_kp = Keypair()
customer_kp = Keypair()
oracle_kp = Keypair()
exchange_booth_kp = Keypair()


class InitExchangeBoothParams(NamedTuple):
    program_id: PublicKey
    admin_kp: Keypair  # [S]
    exchange_booth: PublicKey  # [W]
    oracle: PublicKey
    vault_a: PublicKey  # [W]
    mint_a: PublicKey
    token_a: Token
    vault_b: PublicKey  # [W]
    mint_b: PublicKey
    token_b: Token


class SetExchangeRateParams(NamedTuple):
    program_id: PublicKey
    oracle: PublicKey
    exchange_rate_a_to_b: float


class ExchangeParams(NamedTuple):
    program_id: PublicKey
    amount_to_exchange: int
    exchange_booth: PublicKey
    oracle: PublicKey
    vault_a: PublicKey
    vault_b: PublicKey
    mint_a: PublicKey
    mint_b: PublicKey
    customer_kp: PublicKey
    customer_from_token_account: PublicKey
    customer_to_token_account: PublicKey


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


def exchange_instruction(params: ExchangeParams) -> TransactionInstruction:
    # combine with exchange rate
    data = b"".join([struct.pack("<B", 3), struct.pack("<d", params.amount_to_exchange)])

    return TransactionInstruction(
        keys=[
            AccountMeta(pubkey=params.exchange_booth, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.oracle, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.vault_a, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.vault_b, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.mint_a, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.mint_b, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.customer_kp, is_signer=True, is_writable=False),
            AccountMeta(pubkey=params.customer_from_token_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.customer_to_token_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        program_id=params.program_id,
        data=data,
    )

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
            AccountMeta(pubkey=params.admin_kp.public_key, is_signer=True, is_writable=False),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=SYSVAR_RENT_PUBKEY, is_signer=False, is_writable=False),
        ],
        program_id=params.program_id,
        data=data,
    )

def init(program_id, client) -> CommandParams:
    program_id = PublicKey(program_id)

    ixs = []
    # create accounts and allocate space
    for _account, _space in [(oracle_kp, 16), (exchange_booth_kp, 32 * 4)]:
        ixs.append(
            create_account(
                CreateAccountParams(
                    from_pubkey=admin_kp.public_key,
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
        admin_kp,
        admin_kp.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint_a = token_a.pubkey

    token_b = Token.create_mint(
        client,
        admin_kp,
        admin_kp.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint_b = token_b.pubkey

    # create PDA for 'vault_a'
    vault_a, _ = PublicKey.find_program_address(
        [
            b"exchange_booth",
            bytes(admin_kp.public_key),
            bytes(exchange_booth_kp.public_key),
            bytes(mint_a),
        ],
        program_id,
    )

    # create PDA for 'vault_b'
    vault_b, _ = PublicKey.find_program_address(
        [
            b"exchange_booth",
            bytes(admin_kp.public_key),
            bytes(exchange_booth_kp.public_key),
            bytes(mint_b),
        ],
        program_id,
    )

    params = InitExchangeBoothParams(
        program_id=program_id,
        admin_kp=admin_kp,
        exchange_booth=exchange_booth_kp.public_key,
        oracle=oracle_kp.public_key,
        vault_a=vault_a,
        mint_a=mint_a,
        token_a=token_a,
        vault_b=vault_b,
        mint_b=mint_b,
        token_b=token_b,
    )

    ixs.append(init_exchange_booth(params))
    signers = [admin_kp, oracle_kp, exchange_booth_kp]

    return CommandParams(instructions=ixs, signers=signers, params=params)


def set_rate(
    program_id,
    client,
    exchange_rate_a_to_b: float,
    oracle: Optional[PublicKey] = None
):
    program_id = PublicKey(program_id)
    ixs = []
    signers = []

    if oracle is None:
        print("Creating oracle account because init didn't run")
        ixs.append(
            create_account(
                CreateAccountParams(
                    from_pubkey=admin_kp.public_key,
                    new_account_pubkey=oracle_kp.public_key,
                    lamports=client.get_minimum_balance_for_rent_exemption(40)[
                        "result"
                    ],
                    space=16,
                    program_id=program_id,
                )
            )
        )
        signers.append(oracle_kp)
        oracle = oracle_kp.public_key

    params = SetExchangeRateParams(
        program_id=program_id,
        oracle=oracle,
        exchange_rate_a_to_b=exchange_rate_a_to_b,
    )

    ixs.append(set_exchange_rate(params))
    signers.append(admin_kp)

    return CommandParams(instructions=ixs, signers=signers, params=params)

def deposit(
    program_id, 
    client,
    amount: int, 
    chosen_token,
    init_params: InitExchangeBoothParams
):

    token_to_deposit = init_params.token_a if chosen_token == 'a' else init_params.token_b
    vault_to_deposit = init_params.vault_a if chosen_token == 'a' else init_params.vault_b

    print(f'A={init_params.vault_a}, B={init_params.vault_b}, vault_to_deposit={vault_to_deposit}')

    Token.mint_to(
        token_to_deposit,
        PublicKey(vault_to_deposit),
        admin_kp,
        amount*1000000
    )

def exchange(
    program_id, 
    client, 
    amount_to_exchange: int, 
    token_to_exchange,
    admin_kp: Optional[Keypair] = None,
    exchange_booth: Optional[PublicKey] = None,
    oracle: Optional[PublicKey] = None,
    vault_a: Optional[PublicKey] = None,
    mint_a: Optional[PublicKey] = None,
    token_a: Token = None,
    vault_b: Optional[PublicKey] = None,
    mint_b: Optional[PublicKey] = None,
    token_b: Token = None,
):
    
    print("Requesting Airdrop of 2 SOL For Customer...")
    client.request_airdrop(customer_kp.public_key, int(2e9))
    print("Airdrop received")

    from_token = token_a if token_to_exchange == 'a' else token_b
    to_token = token_b if token_to_exchange == 'a' else token_a

    customer_from_token_account: PublicKey = from_token.create_account(customer_kp.public_key)
    customer_to_token_account: PublicKey = to_token.create_account(customer_kp.public_key)

    Token.mint_to(
        from_token,
        customer_from_token_account,
        admin_kp,
        amount_to_exchange
    )

    program_id = PublicKey(program_id)

    ixs = []
    signers = []
    params = ExchangeParams(
        program_id=program_id,
        amount_to_exchange=amount_to_exchange,
        exchange_booth=exchange_booth,
        oracle=oracle,
        vault_a=vault_a,
        vault_b=vault_b,
        mint_a=mint_a,
        mint_b=mint_b,
        customer_kp=customer_kp.public_key,
        customer_from_token_account=customer_from_token_account,
        customer_to_token_account=customer_to_token_account,
    )
    ixs.append(exchange_instruction(params))
    signers = [customer_kp]

    return CommandParams(instructions=ixs, signers=signers, params=params)


def main():
    client = Client("https://api.devnet.solana.com")
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "program_id",
        help="Devnet program ID (base58 encoded string) of the deployed Echo Program",
    )
    args = parser.parse_args()
    ixs_supported = ("init", "set_rate", "exchange", "deposit", "exit")

    command_params = {}

    print("Requesting Airdrop of 2 SOL...")
    client.request_airdrop(admin_kp.public_key, int(2e9))
    print("Airdrop received")

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

        elif command_input == "exchange":
            _from_token = input("Please enter token to exchange:\n")
            _amount = int(input("Enter amount:\n"))
            _params = exchange(
                args.program_id,
                client,
                _amount,
                _from_token,
                command_params["init"].params.admin_kp,
                command_params["init"].params.exchange_booth,
                command_params["init"].params.oracle,
                command_params["init"].params.vault_a,
                command_params["init"].params.mint_a,
                command_params["init"].params.token_a,
                command_params["init"].params.vault_b,
                command_params["init"].params.mint_b,
                command_params["init"].params.token_b,
            )

            command_params["exchange"] = _params
        
        elif command_input == "deposit":
            _token_to_deposit = input("Please enter token to deposit:\n")
            _amount = int(input("Enter amount:\n"))

            _params = deposit(
                args.program_id,
                client,
                _amount,
                _token_to_deposit,
                command_params["init"].params
            )

        elif command_input == "exit":
            sys.exit(0)

        else:
            raise RuntimeError(f"{command_input} not supported yet")

        if command_input != "deposit":
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
