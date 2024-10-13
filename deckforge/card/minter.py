import json
import random
from deckforge.card.TradingCard import TradingCard
from deckforge.blockchain.core import Blockchain

class CardMinter:
    def __init__(self, metadata_file, blockchain_file='blockchain.json'):
        with open(metadata_file, 'r') as file:
            self.card_metadata = json.load(file)
        self.blockchain = Blockchain(blockchain_file)

    def mint_card(self, wallet_owner):
        rarity = self._select_rarity()
        card_data = self._select_card_data(rarity)
        foil = random.random() < 0.10
        fullart = random.random() < 0.02
       
        if not fullart:
            borderless = random.random() < 0.05    
        else:
            borderless = False
        
        new_card = TradingCard(card_data['title'], card_data['description'], rarity, foil, fullart, borderless)
        self.blockchain.write_card(new_card, wallet_owner)
        return new_card

    def _select_rarity(self):
        roll = random.random()
        if roll < 0.70:
            return 'common'
        elif roll < 0.95:
            return 'uncommon'
        else:
            return 'rare'

    def _select_card_data(self, rarity):
        cards_of_rarity = [card for card in self.card_metadata if card['rarity'] == rarity]
        return random.choice(cards_of_rarity)