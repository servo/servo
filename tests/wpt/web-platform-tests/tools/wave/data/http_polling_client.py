from .client import Client


class HttpPollingClient(Client):
    def __init__(self, session_token, event):
        super().__init__(session_token)
        self.event = event

    def send_message(self, message):
        self.message = message
        self.event.set()
