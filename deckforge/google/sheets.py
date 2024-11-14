import gspread

from gspread.utils import GridRangeType

from deckforge.core.config import get_google_sheet_name

def get_sheet_service():
    return gspread.service_account(filename='service_account.json')

def retrieve_card_metadata():
    sheet_name = get_google_sheet_name()
    service = get_sheet_service()

    card_metadata = []
    card_num = 0
    for sheet in service.open(sheet_name).worksheets():
        cell_values = sheet.get(return_type=GridRangeType.ListOfLists, maintain_size=True)
        cell_values.pop(0)
        
        for row in standardize_metadata_rows(cell_values):
            card_num += 1
            card_metadata.append({
                'id': row[1],
                'section_sort': row[2],
                'card_num': card_num,
                'title': row[3],
                'type': row[4],
                'description': row[5],
                'rarity': row[6],
                'text_ready': row[7],
                'image_ready': row[8],
                'notes': row[10],
                'base_art': '',
                "reference_images": []
            })

    return card_metadata

def standardize_metadata_rows(rows):
    for row in rows:
        for i in range(len(row)):
            if row[i] == '-':
                row[i] = None
            if row[i] == '':
                row[i] = None
            if row[i] == 'TRUE':
                row[i] = True
            if row[i] == 'FALSE':
                row[i] = False
    return rows

