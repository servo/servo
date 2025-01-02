# mypy: allow-untyped-defs

webdriver = None


def do_delayed_imports():
    global webdriver
    import webdriver


class BidiBluetoothSimulateAdapterAction:
    name = "bidi.bluetooth.simulate_adapter"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        if payload["context"] is None:
            raise ValueError("Missing required parameter: context")

        context = payload["context"]
        if isinstance(context, str):
            pass
        elif isinstance(context, webdriver.bidi.protocol.BidiWindow):
            # Context can be a serialized WindowProxy.
            context = context.browsing_context
        else:
            raise ValueError("Unexpected context type: %s" % context)

        state = payload["state"]
        return await self.protocol.bidi_bluetooth.simulate_adapter(context, state)


class BidiSessionSubscribeAction:
    name = "bidi.session.subscribe"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        events = payload["events"]
        contexts = None
        if payload["contexts"] is not None:
            contexts = []
            for context in payload["contexts"]:
                # Context can be either a browsing context id, or a BiDi serialized window. In the latter case, the
                # value is extracted from the serialized object.
                if isinstance(context, str):
                    contexts.append(context)
                elif isinstance(context, webdriver.bidi.protocol.BidiWindow):
                    contexts.append(context.browsing_context)
                else:
                    raise ValueError("Unexpected context type: %s" % context)
        return await self.protocol.bidi_events.subscribe(events, contexts)


class BidiPermissionsSetPermissionAction:
    name = "bidi.permissions.set_permission"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        descriptor = payload['descriptor']
        state = payload['state']
        origin = payload['origin']
        return await self.protocol.bidi_permissions.set_permission(descriptor,
                                                                   state,
                                                                   origin)


async_actions = [
    BidiBluetoothSimulateAdapterAction,
    BidiPermissionsSetPermissionAction,
    BidiSessionSubscribeAction]
