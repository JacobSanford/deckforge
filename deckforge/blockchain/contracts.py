import time
import random

class SmartContract:
    @staticmethod
    def mint_card(blockchain, owner: str):
        all_cards_metadata = blockchain.execute_smart_contract('cards_metadata')
        card_rarity = blockchain.execute_smart_contract('card_rarity')
        cards_of_rarity = [card for card in all_cards_metadata if card['rarity'] == card_rarity]
        base_card = random.choice(cards_of_rarity)
        foil, fullart, borderless = blockchain.execute_smart_contract('card_properties')
        card_data = {
            'id': blockchain.hash_block(owner, len(blockchain.chain), str(time.time())),
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
            'data': card_data
        }
        blockchain.add_block([transaction])

        # Add a transfer transaction to transfer the card to the owner
        transfer_transaction = {
            'type': 'transfer_card',
            'timestamp': str(time.time()),
            'data': {
                'card_id': card_data['id'],
                'from': 'system',
                'to': owner
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

    @staticmethod
    def from_code(code):
        """
        Convert a smart contract from bytecode.
        """
        return SmartContract.__code__.replace(co_code=bytes.fromhex(code))
