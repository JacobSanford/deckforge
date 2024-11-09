from deckforge.blockchain.crypto import generate_keypair, pubkey_to_pubaddress

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

    private_key = None
    public_key = None
    address = None

    def __init__(self):
        self.private_key, self.public_key = generate_keypair()
        self.address = pubkey_to_pubaddress(self.public_key)
