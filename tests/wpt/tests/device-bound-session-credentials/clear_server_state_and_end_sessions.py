import importlib
session_provider = importlib.import_module('device-bound-session-credentials.session_provider')

def main(request, response):
    session_provider.clear_server_state()
    return (200, [("Clear-Site-Data", '"cookies"')], "")
