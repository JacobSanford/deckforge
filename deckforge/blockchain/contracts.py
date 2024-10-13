import hashlib
import time

class SmartContract:
    @staticmethod
    def mint_card(card_data, wallet_owner):
        """
        Pseudo smart contract method to mint a card and return a transaction.
        """
        card_id = hashlib.sha256(f"{card_data['title']}{card_data['description']}{card_data['rarity']}{time.time()}".encode()).hexdigest()
        transaction = {
            'type': 'mint_card',
            'timestamp': str(time.time()),
            'wallet_owner': wallet_owner,
            'card_id': card_id,
            'data': card_data
        }
        return transaction
