import deckforge.blockchain.crypto

def generate_wallet():
    """
    Generate a new wallet.

    Returns:
        dict: The wallet.
    """
    private_key, public_key = deckforge.blockchain.crypto.generate_keypair()
    return {
        'private_key': private_key,
        'public_key': public_key,
        'address': deckforge.blockchain.crypto.pubkey_to_pubaddress(public_key)
    }
