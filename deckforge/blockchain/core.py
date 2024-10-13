import json
import os
import hashlib
import time
import deckforge.card.TradingCard as TradingCard

from deckforge.blockchain.contracts import SmartContract

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
                'transactions': [],
                'smart_contracts': self.load_smart_contracts()
            }
            self.chain.append(genesis_block)
            self.save_chain()

    def load_smart_contracts(self):
        """
        Loads the initial smart contracts as a dictionary of name to code.
        """
        return {
            'mint_card': SmartContract.mint_card.__code__.co_code.hex()
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
        """
        Executes a smart contract method stored in any block.
        """
        for block in self.chain:
            if 'smart_contracts' in block and contract_name in block['smart_contracts']:
                smart_contract_code = bytes.fromhex(block['smart_contracts'][contract_name])
                exec(smart_contract_code, globals())
                method = globals()[contract_name]
                return method(*args, **kwargs)
        raise ValueError(f"Smart contract {contract_name} not found in the blockchain.")

    def register_smart_contract(self, contract_name, contract_code):
        """
        Registers a new smart contract by adding it to a new block.
        """
        smart_contracts = {
            contract_name: contract_code.hex()
        }
        self.add_block([], smart_contracts)

    def write_card(self, card: TradingCard, wallet_owner: str):
        """
        Writes a minted TradingCard to the blockchain using the smart contract.
        """
        card_data = {
            'title': card.title,
            'description': card.description,
            'rarity': card.rarity,
            'isFoil': card.isFoil,
            'isBorderless': card.isBorderless,
            'isFullart': card.isFullart
        }
        transaction = self.execute_smart_contract('mint_card', card_data, wallet_owner)
        self.add_block([transaction])
