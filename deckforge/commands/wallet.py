from deckforge.wallet.wallet import create_wallet

def generate_wallet():
    """
    Generate a new wallet.

    Returns:
        dict: The wallet.
    """
    wallet = create_wallet()
    print(f"Private key: {wallet.private_key}")
    print(f"Public key: {wallet.public_key}")
    print(f"Public address: {wallet.address}")
