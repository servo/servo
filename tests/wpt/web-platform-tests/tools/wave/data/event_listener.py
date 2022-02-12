class EventListener(object):
    def __init__(self, dispatcher_token):
        super(EventListener, self).__init__()
        self.dispatcher_token = dispatcher_token
        self.token = None

    def send_message(self, message):
        raise Exception("Client.send_message(message) not implemented!")
