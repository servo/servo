STATUS_EVENT = "status"
RESUME_EVENT = "resume"
TEST_COMPLETED_EVENT = "test_completed"


class EventDispatcher(object):
    def __init__(self):
        self._clients = {}

    def add_session_client(self, client):
        token = client.session_token
        if token not in self._clients:
            self._clients[token] = []
        self._clients[token].append(client)

    def remove_session_client(self, client_to_delete):
        if client_to_delete is None:
            return
        token = client_to_delete.session_token
        if token not in self._clients:
            return

        for client in self._clients[token]:
            if client.session_token == client_to_delete.session_token:
                self._clients.remove(client)
                break
        if len(self._clients[token]) == 0:
            del self._clients[token]

    def dispatch_event(self, token, event_type, data):
        if token not in self._clients:
            return
        event = {
            "type": event_type,
            "data": data
        }

        for client in self._clients[token]:
            client.send_message(event)
