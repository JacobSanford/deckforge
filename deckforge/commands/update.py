import gspread
import json

from gspread.utils import GridRangeType
from deckforge.core.path import get_data_file_path

def get_latest_card_metadata():
    """
    Helper command: fetch the latest card metadata from the Google Sheet.

    The output can be pasted into the card metadata smart contract.
    """
    creds_file = '/home/jsanford/gitDev/deckforge/service_account.json'
    sheet_name = 'Happy Times Trading Cards'
    gc = gspread.service_account(filename=creds_file)

    card_metadata = []
    card_id = 0
    for sheet in gc.open(sheet_name).worksheets():
        cell_values = sheet.get(return_type=GridRangeType.ListOfLists, maintain_size=True)
        cell_values.pop(0) # Remove the header row
        
        for row in cell_values:
            card_id += 1
            for i in range(len(row)):
                if row[i] == '-':
                    row[i] = None

            card_metadata.append({
                'id': row[1],
                'section_sort': row[2],
                'card_num': card_id,
                'title': row[3],
                'type': row[4],
                'description': row[5],
                'rarity': row[6],
                'text_ready': False,
                'image_ready': False,
                'base_art': '',
                "reference_images": [],
                'notes': ''
            })

    with open(get_data_file_path('card_metadata.json'), 'w') as f:
        f.write(json.dumps(card_metadata, indent=4))
