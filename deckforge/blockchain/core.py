import json
import os
import hashlib
import time

from binascii import unhexlify
from deckforge.blockchain.contracts import SmartContract

class Blockchain:
    NULL_ADDRESS = '0x0000000000000000000000000000000000000000'

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
                'transactions': [],
                'mint_limit': 5000,
            }
            self.chain.append(genesis_block)
            self.save_chain()

    def genesis_smart_contracts(self):
        """
        Loads the initial smart contracts.
        """
        return {
            'mint_card': SmartContract.mint_card.__code__.co_code.hex(),
            'cards_metadata': SmartContract.cards_metadata.__code__.co_code.hex(),
            'card_rarity': SmartContract.card_rarity.__code__.co_code.hex(),
            'card_properties': SmartContract.card_properties.__code__.co_code.hex(),
        }

    def hash_block(self, previous_hash, index, timestamp):
        block_string = f"{previous_hash}{index}{timestamp}"
        return hashlib.sha256(block_string.encode()).hexdigest()

    def add_block(self, transactions, smart_contracts=None):
        previous_block = self.chain[-1]
        new_block = {
            'index': len(self.chain),
            'timestamp': str(time.time()),
            'previous_hash': previous_block['hash'],
            'hash': self.hash_block(previous_block['hash'], len(self.chain), str(time.time())),
            'transactions': transactions,
            'smart_contracts': smart_contracts or {}
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

    def execute_smart_contract(self, contract_name, *args, **kwargs):
        # Run smart contracts diretly from the SmartContract class
        if hasattr(SmartContract, contract_name):
            return getattr(SmartContract, contract_name)(*args, **kwargs)
        raise ValueError(f"Smart contract {contract_name} not found in the blockchain.")

    def register_smart_contract(self, contract_name, contract_code):
        """
        Registers a new smart contract by adding it to a new block.
        """
        smart_contracts = {
            contract_name: contract_code.hex()
        }
        self.add_block([], smart_contracts)

    def card_exists(self, card_id):
        for block in self.chain:
            for transaction in block['transactions']:
                if transaction['type'] == 'mint_card' and transaction['data']['id'] == card_id:
                    return True
        return False
    
    def get_card_wallet(self, card_id):
        for block in self.chain:
            for transaction in block['transactions']:
                if transaction['type'] == 'transfer_card' and transaction['data']['card_id'] == card_id:
                    return transaction['data']['to']
        return None