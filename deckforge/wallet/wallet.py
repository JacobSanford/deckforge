# This file contains the Wallet class, which is responsible for the creation and management of wallets.

from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.backends import default_backend

def create_wallet():
    """
    Creates a new wallet and returns the wallet object.
    """
    wallet = Wallet()
    return wallet

class Wallet:
    """
    Represents a wallet.
    """

    def __init__(self):
        self.private_key = self.generate_private_key()
        self.public_key = self.generate_public_key()

    def generate_private_key(self):
        """
        Generates a private key.
        """
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048,
            backend=default_backend()
        )
        return private_key

    def generate_public_key(self):
        """
        Generates a public key.
        """
        public_key = self.private_key.public_key()
        return public_key

    def private_key_to_pem(self):
        """
        Serializes the private key to PEM format.
        """
        pem = self.private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.TraditionalOpenSSL,
            encryption_algorithm=serialization.NoEncryption()
        )
        return pem

    def public_key_to_pem(self):
        """
        Serializes the public key to PEM format.
        """
        pem = self.public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )
        return pem


