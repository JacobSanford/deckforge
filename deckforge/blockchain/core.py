import json
import os
import hashlib
import time

import deckforge.card.TradingCard as TradingCard

class Blockchain:
    def __init__(self, file_path='blockchain.json'):
        self.file_path = file_path
        self.chain = []
        self.create_genesis_block()
        self.load_chain()

    def create_genesis_block(self):
        if not os.path.exists(self.file_path):
            genesis_block = {
                'index': 0,
                'timestamp': str(time.time()),
                'previous_hash': '0',
                'hash': self.hash_block('0', 0, str(time.time())),
                'transactions': []
            }
            self.chain.append(genesis_block)
            self.save_chain()

    def hash_block(self, previous_hash, index, timestamp):
        block_string = f"{previous_hash}{index}{timestamp}"
        return hashlib.sha256(block_string.encode()).hexdigest()

    def add_block(self, transactions):
        previous_block = self.chain[-1]
        new_block = {
            'index': len(self.chain),
            'timestamp': str(time.time()),
            'previous_hash': previous_block['hash'],
            'hash': self.hash_block(previous_block['hash'], len(self.chain), str(time.time())),
            'transactions': transactions
        }
        self.chain.append(new_block)
        self.save_chain()

    def save_chain(self):
        with open(self.file_path, 'w') as f:
            json.dump(self.chain, f, indent=4)

    def load_chain(self):
        if os.path.exists(self.file_path):
            with open(self.file_path, 'r') as f:
                self.chain = json.load(f)

    def mint_card_to_blockchain(self, card: TradingCard):
        """
        Writes a minted TradingCard to the blockchain.
        """
        card_data = {
            'title': card.title,
            'description': card.description,
            'rarity': card.rarity,
            'isFoil': card.isFoil,
            'isBorderless': card.isBorderless,
            'isFullart': card.isFullart
        }
        
        # Generate a secure card ID
        card_id = hashlib.sha256(f"{card.title}{card.description}{card.rarity}{time.time()}".encode()).hexdigest()
        
        transaction = {
            'card_id': card_id,
            'type': 'mint',
            'timestamp': str(time.time()),
            'data': card_data
        }
        self.add_block([transaction])

    def transact_card_to_blockchain(self, card_id: str, sender: str, recipient: str):
        """
        Writes a transaction of a TradingCard to the blockchain.
        """
        transaction = {
            'card_id': card_id,
            'type': 'transfer',
            'timestamp': str(time.time()),
            'sender': sender,
            'recipient': recipient,
        }
        self.add_block([transaction])
