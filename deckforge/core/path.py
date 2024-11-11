import os

from pathlib import Path

def get_repo_path():
    """
    Get the path to the root of the repository.
    """
    return Path(__file__).parent.parent.parent

def get_data_path():
    """
    Get the path to the data directory.
    """
    return os.path.join(get_repo_path(), 'data')

def get_data_file_path(file_name):
    """
    Get the path to the data file.
    """
    return os.path.join(get_data_path(), file_name)