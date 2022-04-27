from .event_listener import EventListener

class HttpPollingEventListener(EventListener):
    def __init__(self, dispatcher_token, event):
        super().__init__(dispatcher_token)
        self.event = event
        self.message = None

    def send_message(self, message):
        self.message = message
        self.event.set()
