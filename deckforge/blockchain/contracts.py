import time
import random

from deckforge.blockchain.crypto import extract_pubkey_from_sig, valid_secp256k1_pubkey, pubkey_to_pubaddress, validate_signature, check_wallet_checksum

class SmartContract:
    @staticmethod
    def mint_card(blockchain, wallet_address: str):
        # Check if the wallet address is valid.
        if check_wallet_checksum(wallet_address) == False:
            raise ValueError("Invalid wallet address.")

        # Check if the maximum card count has been reached.
        cards_remaining = blockchain.execute_smart_contract('get_cards_remaining', blockchain)
        if cards_remaining <= 0:
            raise ValueError("Maximum card count reached. No more cards can be minted.")

        all_cards_metadata = blockchain.execute_smart_contract('cards_metadata')
        card_rarity = blockchain.execute_smart_contract('card_rarity')
        cards_of_rarity = [card for card in all_cards_metadata if card['rarity'] == card_rarity]
        base_card = random.choice(cards_of_rarity)
        foil, fullart, borderless = blockchain.execute_smart_contract('card_properties')
        card_data = {
            'id': blockchain.hash_block(wallet_address, len(blockchain.chain), str(time.time())),
            'title': base_card['title'],
            'description': base_card['description'],
            'rarity': card_rarity,
            'isFoil': foil,
            'isBorderless': borderless,
            'isFullart': fullart
        }
        transaction = {
            'type': 'mint_card',
            'timestamp': str(time.time()),
            'data': card_data,
        }
        blockchain.add_block([transaction])
        blockchain.execute_smart_contract('transfer_card', blockchain, card_data['id'], wallet_address)

    @staticmethod
    def get_cards_remaining(blockchain):
        if blockchain.chain[0].get('mint_limit') == None:
            raise ValueError(f"Error: Mint total not found in the genesis block.")
        minted_cards = 0
        for block in blockchain.chain:
            for transaction in block['transactions']:
                if transaction['type'] == 'mint_card':
                    minted_cards += 1
        cards_remaining = blockchain.chain[0]['cards_remaining'] - minted_cards
        if cards_remaining < 0:
            raise ValueError(f"Error: Negative cards remaining.")
        return cards_remaining

    @staticmethod
    def transfer_card(blockchain, card_id: str, to_address: str, signature: str=''):
        if not blockchain.card_exists(card_id):
            raise ValueError(f"Card with ID {card_id} does not exist.")

        owner_address = blockchain.get_card_wallet(card_id)
        if owner_address == None:
            # This is a minted card.
            owner_address = blockchain.NULL_ADDRESS
        else:
            if not signature:
                raise ValueError("Signature is required for transferring a card.")

            sig_pubkey = extract_pubkey_from_sig(signature, card_id)
            sig_address = pubkey_to_pubaddress(sig_pubkey)
            if sig_address != owner_address:
                raise ValueError("Invalid signature owner for transferring card.")

            sig_is_valid = validate_signature(signature, sig_pubkey, to_address)
            if not sig_is_valid:
                raise ValueError("Invalid signature for transferring card.")

        # Transaction can proceed.
        transfer_transaction = {
            'type': 'transfer_card',
            'timestamp': str(time.time()),
            'data': {
                'card_id': card_id,
                'from': owner_address,
                'to': to_address
            }
        }
        blockchain.add_block([transfer_transaction])

    @staticmethod
    def card_properties():
        """
        Pseudo smart contract method to determine the properties of a card.
        """
        foil = random.random() < 0.10
        fullart = random.random() < 0.02
       
        if not fullart:
            borderless = random.random() < 0.05    
        else:
            borderless = False
        return foil, fullart, borderless

    @staticmethod
    def card_rarity():
        """
        Pseudo smart contract method to determine the rarity of a card.
        """
        roll = random.random()
        if roll < 0.70:
            return 'common'
        elif roll < 0.95:
            return 'uncommon'
        else:
            return 'rare'

    @staticmethod
    def cards_metadata():
        """
        Pseudo smart contract method to return all card metadata.
        """
        return [
            {
                "id": 1,
                "title": "Flame Dragon",
                "description": "A powerful dragon with the ability to breathe fire.",
                "rarity": "rare"
            },
            {
                "id": 2,
                "title": "Mystic Elf",
                "description": "An elf with mystical powers and high intelligence.",
                "rarity": "uncommon"
            },
            {
                "id": 3,
                "title": "Warrior Knight",
                "description": "A brave knight skilled in combat.",
                "rarity": "common"
            },
            {
                "id": 4,
                "title": "Shadow Assassin",
                "description": "A stealthy assassin who strikes from the shadows.",
                "rarity": "rare"
            },
            {
                "id": 5,
                "title": "Healing Priest",
                "description": "A priest with the ability to heal allies.",
                "rarity": "uncommon"
            },
            {
                "id": 6,
                "title": "Thunder Giant",
                "description": "A giant with the power to summon thunderstorms.",
                "rarity": "rare"
            },
            {
                "id": 7,
                "title": "Forest Guardian",
                "description": "A guardian who protects the forest with nature magic.",
                "rarity": "uncommon"
            },
            {
                "id": 8,
                "title": "Ice Sorceress",
                "description": "A sorceress who controls ice and snow.",
                "rarity": "rare"
            },
            {
                "id": 9,
                "title": "Goblin Thief",
                "description": "A sneaky goblin who steals from enemies.",
                "rarity": "common"
            },
            {
                "id": 10,
                "title": "Stone Golem",
                "description": "A golem made of stone with immense strength.",
                "rarity": "common"
            }
        ]

