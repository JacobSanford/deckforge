from deckforge.google.auth import get_drive_service

# constant values
happytimes_folder_id = '1bkMrDNU7LuqjlxRKVOtRLudTaDNVnD9V'

def test_google_drive():
    service = get_drive_service()

    # Create a folder if it does not exist.
    file_metadata = {
        'name': 'test',
        'mimeType': 'application/vnd.google-apps.folder',
        'parents': [happytimes_folder_id],
    }

    # Check if the folder already exists
    results = service.files().list(q=f"name='test' and '{happytimes_folder_id}' in parents").execute()
    items = results.get('files', [])
    if items:
        print('Folder already exists')
        return
    # Create it.
    file = service.files().create(body=file_metadata, fields='id').execute()

