import json
import os

from deckforge.core.path import get_repo_path

CONFIG_FILE = 'deckforge.json'

def get_config_item(item_key,):
    config_path = os.path.join(get_repo_path(), CONFIG_FILE)
    
    if not os.path.exists(config_path):
        raise FileNotFoundError(f"Config file {CONFIG_FILE} not found.")
    
    with open(config_path, 'r') as file:
        config_data = json.load(file)
    
    if item_key not in config_data:
        raise KeyError(f"Key {item_key} not found in config file.")
    
    return config_data[item_key]

def get_google_sheet_name():
    return get_config_item('google-sheet-name')

def get_google_root_folder_id():
    return get_config_item('google-drive-root-folder-id')
