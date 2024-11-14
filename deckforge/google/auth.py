from oauth2client.service_account import ServiceAccountCredentials

def google_api_auth(scopes):
    creds = None
    creds = ServiceAccountCredentials.from_json_keyfile_name('service_account.json', scopes)
    return creds
