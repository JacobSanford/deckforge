from deckforge.card.core import get_cards

def get_latest_card_metadata():
    """
    Helper command: fetch the latest card metadata from the Google Sheet.

    The output can be pasted into the card metadata smart contract.
    """
    cards = get_cards()
    for card in cards:
        print(card)
