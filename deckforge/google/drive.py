from deckforge.google.auth import google_api_auth
from googleapiclient.discovery import build

SCOPES = ['https://www.googleapis.com/auth/drive.metadata', 'https://www.googleapis.com/auth/drive']


CARD_DATA_FOLDER_NAME = 'Card Data'
CARD_DATA_FOLDER_ID = '15PVZS7dlL_xDOlL7_SrSHDgD9YLnHfPz'
ROOT_FOLDER_ID = '1bkMrDNU7LuqjlxRKVOtRLudTaDNVnD9V'

def get_drive_service():
    creds = google_api_auth(SCOPES)
    service = build('drive', 'v3', credentials=creds)
    check_create_root_data_folder(service)
    return service

def check_create_root_data_folder(service):
    results = service.files().list(q=f"name='{CARD_DATA_FOLDER_NAME}' and '{ROOT_FOLDER_ID}' in parents and mimeType='application/vnd.google-apps.folder'").execute()
    items = results.get('files', [])
    if not items:
        file_metadata = {
            'name': CARD_DATA_FOLDER_NAME,
            'parents': [ROOT_FOLDER_ID],
            'mimeType': 'application/vnd.google-apps.folder'
        }
        service.files().create(body=file_metadata).execute()

def check_create_card_data_folder(service, card):
    results = service.files().list(q=f"name='{card.id}' and '{CARD_DATA_FOLDER_ID}' in parents").execute()
    items = results.get('files', [])
    if not items:
        file_metadata = {
            'name': card.id,
            'parents': [CARD_DATA_FOLDER_ID],
            'mimeType': 'application/vnd.google-apps.folder'
        }
        print(f"Creating folder for {card}")
        service.files().create(body=file_metadata).execute()
