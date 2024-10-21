import gspread
import json

from gspread.utils import GridRangeType

def get_latest_card_metadata():
    """
    Helper command: fetch the latest card metadata from the Google Sheet.

    The output can be pasted into the card metadata smart contract.
    """
    creds_file = '/home/jsanford/gitDev/deckforge/service_account.json'
    sheet_name = 'Happy Times Trading Cards'
    gc = gspread.service_account(filename=creds_file)

    wks = gc.open(sheet_name).sheet1
    cell_values = wks.get(return_type=GridRangeType.ListOfLists, maintain_size=True)
    cell_values.pop(0)

    card_metadata = []
    for row in cell_values:
        for i in range(len(row)):
            if row[i] == '-':
                row[i] = None

        card_metadata.append({
            'id': int(row[0] or 0),
            'title': row[1],
            'type': row[2],
            'description': row[3],
            'rarity': row[4],
            'text_ready': False,
            'image_ready': False,
            'base_art': '',
            'notes': ''
        })
    
    # Output this in pretty json format
    print (json.dumps(card_metadata, indent=4))
