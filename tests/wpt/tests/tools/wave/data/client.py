# mypy: allow-untyped-defs

class Client:
    def __init__(self, session_token):
        self.session_token = session_token

    def send_message(self, message):
        raise Exception("Client.send_message(message) not implemented!")
