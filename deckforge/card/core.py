from deckforge.google.sheets import retrieve_card_metadata
from deckforge.google.drive import get_drive_service, check_create_card_data_folder

image_paths = {
    'base_art': 'base_art',
    'reference_images': 'reference_images'
}

class TradingCard:
    def __init__(self, card_metadata):
        self.id = card_metadata['id']
        self.section_sort = card_metadata['section_sort']
        self.card_num = card_metadata['card_num']
        self.title = card_metadata['title']
        self.type = card_metadata['type']
        self.description = card_metadata['description']
        self.rarity = card_metadata['rarity']
        self.text_ready = card_metadata['text_ready']
        self.image_ready = card_metadata['image_ready']
        self.notes = card_metadata['notes']
        self.base_art = card_metadata['base_art']
        self.reference_images = card_metadata['reference_images']

    def __str__(self):
        return f"{self.title} ({self.id})"

def get_card_metadata():
    card_metadata = retrieve_card_metadata()
    return [TradingCard(card) for card in card_metadata]

def get_cards():
    '''
    Ultimately these methods should be storage-agnostic and use a configurable adapter. But I'm lazy.
    '''
    cards = get_card_metadata()
    service = get_drive_service()
    for card in cards:
        check_create_card_data_folder(service, card)
    return cards