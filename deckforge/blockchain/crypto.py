import binascii

import web3

# Import ec
from cryptography.hazmat.primitives.asymmetric import ec

from ecdsa import SigningKey, VerifyingKey, SECP256k1
from ecdsa.util import sigdecode_der

@staticmethod
def valid_secp256k1_pubkey(public_key: str | bytes) -> bool:
    """
    Validate a secp256k1 public key.

    Args:
        public_key (str | bytes): The public key to validate.

    Returns:
        bool: True if the public key is valid, False otherwise.
    """
    try:
        if isinstance(public_key, str):
            public_key = binascii.unhexlify(public_key)
        vk = VerifyingKey.from_string(public_key, curve=SECP256k1)
        return True
    except Exception as e:
        return False

@staticmethod
def pubkey_to_pubaddress(public_key: str) -> str:
    """
    Convert a public key to a public address.

    This follows the Ethereum convention of taking the last 20 bytes of the
    keccak256 hash of the public key.

    Args:
        public_key (str | bytes): The public key to convert.

    Returns:
        str: The public address.
    """
    ec = web3.Web3.keccak(binascii.unhexlify(public_key))
    return web3.Web3.to_checksum_address(f"0x{ec.hex()[-40:]}")

@staticmethod
def extract_pubkey_from_sig(signature: str) -> str:
    """
    Extract the public key from a signature.

    Args:
        sig (str): The signature.

    Returns:
        str: The public key.
    """
    sig = sigdecode_der(binascii.unhexlify(signature), SECP256k1)
    vk = VerifyingKey.from_public_point(sig[1], curve=SECP256k1)
    return vk.to_string().hex()

def validate_signature(signature: str, public_key: str, message:str) -> bool:
    """
    Validate a signature.

    Args:
        signature (str): The signature.
        public_key (str): The public key.

    Returns:
        bool: True if the signature is valid, False otherwise.
    """
    vk = VerifyingKey.from_string(binascii.unhexlify(public_key), curve=SECP256k1)
    return vk.verify(binascii.unhexlify(signature), message.encode())

@staticmethod
def extract_message_from_signature(signature: str) -> str:
    """
    Extract the message from a signature.

    Args:
        sig (str): The signature.

    Returns:
        str: The public key.
    """
    sig = sigdecode_der(binascii.unhexlify(signature), SECP256k1)
    return sig[0].decode()

def is_valid_wallet_address(wallet_address: str) -> bool:
    """
    Validate an wallet address.

    Args:
        wallet_address (str): The wallet address to validate.

    Returns:
        bool: True if the wallet address is valid, False otherwise.
    """
    return wallet_address.startswith('0x') and len(wallet_address) == 42

def generate_keypair() -> tuple[str, str]:
    """
    Generate a new keypair.

    Returns:
        tuple[str, str]: The private and public keys.
    """
    priv_key = SigningKey.generate(curve=SECP256k1)
    pub_key = priv_key.get_verifying_key()
    return priv_key.to_string().hex(), pub_key.to_string().hex()
