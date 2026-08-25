import dataclasses

from websockets.exceptions import NegotiationError


class OpExtension:
    name = "x-op"

    def __init__(self, op=None):
        self.op = op

    def decode(self, frame, *, max_size=None):
        return frame  # pragma: no cover

    def encode(self, frame):
        return frame  # pragma: no cover

    def __eq__(self, other):
        return isinstance(other, OpExtension) and self.op == other.op


class ClientOpExtensionFactory:
    name = "x-op"

    def __init__(self, op=None):
        self.op = op

    def get_request_params(self):
        return [("op", self.op)]

    def process_response_params(self, params, accepted_extensions):
        if params != [("op", self.op)]:
            raise NegotiationError()
        return OpExtension(self.op)


class ServerOpExtensionFactory:
    name = "x-op"

    def __init__(self, op=None):
        self.op = op

    def process_request_params(self, params, accepted_extensions):
        if params != [("op", self.op)]:
            raise NegotiationError()
        return [("op", self.op)], OpExtension(self.op)


class NoOpExtension:
    name = "x-no-op"

    def __repr__(self):
        return "NoOpExtension()"

    def decode(self, frame, *, max_size=None):
        return frame

    def encode(self, frame):
        return frame


class ClientNoOpExtensionFactory:
    name = "x-no-op"

    def get_request_params(self):
        return []

    def process_response_params(self, params, accepted_extensions):
        if params:
            raise NegotiationError()
        return NoOpExtension()


class ServerNoOpExtensionFactory:
    name = "x-no-op"

    def __init__(self, params=None):
        self.params = params or []

    def process_request_params(self, params, accepted_extensions):
        return self.params, NoOpExtension()


class Rsv2Extension:
    name = "x-rsv2"

    def decode(self, frame, *, max_size=None):
        assert frame.rsv2
        return dataclasses.replace(frame, rsv2=False)

    def encode(self, frame):
        assert not frame.rsv2
        return dataclasses.replace(frame, rsv2=True)

    def __eq__(self, other):
        return isinstance(other, Rsv2Extension)


class ClientRsv2ExtensionFactory:
    name = "x-rsv2"

    def get_request_params(self):
        return []

    def process_response_params(self, params, accepted_extensions):
        return Rsv2Extension()


class ServerRsv2ExtensionFactory:
    name = "x-rsv2"

    def process_request_params(self, params, accepted_extensions):
        return [], Rsv2Extension()
