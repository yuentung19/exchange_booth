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
#pack_int = lambda i: struct.pack("")

class InitVendingMachineEchoParams(NamedTuple):
    program_id: PublicKey
    buffer: PublicKey
    mint: PublicKey
    payer: PublicKey
    price: int
    buffer_size: int

def init_vending_machine_echo(params: InitVendingMachineEchoParams) -> TransactionInstruction:
    #data = b"".join([struct.pack("<B", 1), pack_int(params.buffer_seed), pack_int(params.buffer_size)])
    data = b"".join([struct.pack("<B", 3), params.price.to_bytes(8, "little"), params.buffer_size.to_bytes(8, "little")])

    return TransactionInstruction(
        keys=[
            AccountMeta(pubkey=params.buffer, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.mint, is_signer=False, is_writable=False),
            AccountMeta(pubkey=params.payer, is_signer=True, is_writable=False),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False)
        ],
        program_id=params.program_id,
        data=data,
    )

class VendingMachineEchoParams(NamedTuple):
    program_id: PublicKey
    buffer: PublicKey
    user: PublicKey
    user_token_account: PublicKey
    mint: PublicKey
    data: str

def vending_machine_echo(params: VendingMachineEchoParams) -> TransactionInstruction:
    data = b"".join([struct.pack("<B", 4), pack_str(params.data)])

    return TransactionInstruction(
        keys=[
            AccountMeta(pubkey=params.buffer, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.user, is_signer=True, is_writable=False),
            AccountMeta(pubkey=params.user_token_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=params.mint, is_signer=False, is_writable=True),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        program_id=params.program_id,
        data=data,
    )



if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("program_id", help="Devnet program ID (base58 encoded string) of the deployed Echo Program")
    parser.add_argument("price", help="Price")
    parser.add_argument("buffer_size", help="Buffer Size")
    parser.add_argument("echo", help="The string to copy on-chain")
    args = parser.parse_args()

    program_id = PublicKey(args.program_id)
    price = int(args.price)
    buffer_size = int(args.buffer_size)

    fee_payer = Keypair()

    client = Client("https://api.devnet.solana.com")
    print("Requesting Airdrop of 2 SOL...")
    client.request_airdrop(fee_payer.public_key, int(2e9))
    print("Airdrop received")
    
    payer = Keypair()
    token = Token.create_mint(
        client,
        fee_payer,
        fee_payer.public_key,
        6,
        TOKEN_PROGRAM_ID,
    )
    mint = token.pubkey
    user_token_account = token.create_account(payer.public_key)
    token.mint_to(
        user_token_account,
        fee_payer,
        100
    )

    buffer, bump_seed = PublicKey.find_program_address(
        [
            b"vending_machine",
            bytes(mint),
            price.to_bytes(8, "little")
        ],
        program_id
    )

    create_account_ix = create_account(
        CreateAccountParams(
            from_pubkey=fee_payer.public_key,
            new_account_pubkey=payer.public_key,
            lamports=client.get_minimum_balance_for_rent_exemption(40)[
                "result"
            ],
            space=0,
            program_id=SYS_PROGRAM_ID,
        )
    )

    init_vending_machine_echo_ix = init_vending_machine_echo(
        InitVendingMachineEchoParams(
            program_id=program_id,
            buffer=buffer,
            mint=mint,
            payer=payer.public_key,
            price=price,
            buffer_size=buffer_size,
        )
    )

    vending_machine_echo_ix = vending_machine_echo(
        VendingMachineEchoParams(
            program_id=program_id,
            buffer=buffer,
            user=payer.public_key,
            user_token_account=user_token_account,
            mint=mint,
            data=args.echo
        )
    )

    tx = Transaction().add(create_account_ix).add(init_vending_machine_echo_ix).add(vending_machine_echo_ix)
    signers = [fee_payer, payer]
    result = client.send_transaction(
        tx,
        *signers,
        opts=TxOpts(
            skip_preflight=True,
        ),
    )
    tx_hash = result["result"]
    client.confirm_transaction(tx_hash, commitment="confirmed")

    print(f"https://explorer.solana.com/tx/{tx_hash}?cluster=devnet")
'''
    acct_info = client.get_account_info(buffer.public_key, commitment=Confirmed)
    if acct_info["result"]["value"] is None:
        raise RuntimeError(f"Failed to get account. address={buffer.public_key}")
    data = base64.b64decode(acct_info["result"]["value"]["data"][0]).decode("ascii")
    print("Echo Buffer Text:", data)
'''
