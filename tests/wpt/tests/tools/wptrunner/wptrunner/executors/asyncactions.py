# mypy: allow-untyped-defs
from webdriver.bidi.undefined import UNDEFINED

webdriver = None


def do_delayed_imports():
    global webdriver
    import webdriver


def get_browsing_context_id(context):
    """
    :param context: Either a string representing the browsing context id, or a
    BiDi serialized window proxy object. In the latter case, the value is
    extracted from the serialized object.
    :return: The browsing context id.
    """
    if isinstance(context, str):
        return context
    elif isinstance(context, webdriver.bidi.protocol.BidiWindow):
        # Context can be a serialized WindowProxy.
        return context.browsing_context
    raise ValueError("Unexpected context type: %s" % context)


class BidiBluetoothHandleRequestDevicePrompt:
    name = "bidi.bluetooth.handle_request_device_prompt"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        if "context" not in payload:
            raise ValueError("Missing required parameter: context")

        context = get_browsing_context_id(payload["context"])
        prompt = payload["prompt"]
        accept = payload["accept"]
        device = payload["device"]
        return await self.protocol.bidi_bluetooth.handle_request_device_prompt(context, prompt, accept, device)

class BidiBluetoothSimulateAdapterAction:
    name = "bidi.bluetooth.simulate_adapter"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        if "context" not in payload:
            raise ValueError("Missing required parameter: context")

        context = get_browsing_context_id(payload["context"])

        state = payload["state"]
        return await self.protocol.bidi_bluetooth.simulate_adapter(context,
                                                                   state,
                                                                   type_="create")

class BidiBluetoothSimulatePreconnectedPeripheralAction:
    name = "bidi.bluetooth.simulate_preconnected_peripheral"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        if "context" not in payload:
            raise ValueError("Missing required parameter: context")
        context = get_browsing_context_id(payload["context"])

        address = payload["address"]
        name = payload["name"]
        manufacturer_data = payload["manufacturerData"]
        known_service_uuids = payload["knownServiceUuids"]
        return await self.protocol.bidi_bluetooth.simulate_preconnected_peripheral(
            context, address, name, manufacturer_data, known_service_uuids)


class BidiEmulationSetGeolocationOverrideAction:
    name = "bidi.emulation.set_geolocation_override"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        if "error" in payload and "coordinates" in payload:
            raise ValueError(
                "Params `error` and `coordinates` are mutually exclusive")

        # If `error` is present, set it. Otherwise, do not pass it (error: None).
        # Note, unlike `coordinates`, `error` cannot be `UNDEFINED`. It's either
        # `None` and it's not passed, or some dict value which is passed.
        error = payload['error'] if 'error' in payload else None
        # If `error` is present, do not pass `coordinates` (coordinates: UNDEFINED).
        # Otherwise, remove emulation (coordinates: None).
        coordinates = payload['coordinates'] if 'coordinates' in payload else (
            None if error is None else UNDEFINED)

        if "contexts" not in payload:
            raise ValueError("Missing required parameter: contexts")
        contexts = []
        for context in payload["contexts"]:
            contexts.append(get_browsing_context_id(context))
        if len(contexts) == 0:
            raise ValueError("At least one context must be provided")

        return await self.protocol.bidi_emulation.set_geolocation_override(
            coordinates, error, contexts)


class BidiSessionSubscribeAction:
    name = "bidi.session.subscribe"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        events = payload["events"]
        contexts = None
        if "contexts" in payload and payload["contexts"] is not None:
            contexts = []
            for context in payload["contexts"]:
                contexts.append(get_browsing_context_id(context))
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
    BidiBluetoothHandleRequestDevicePrompt,
    BidiBluetoothSimulateAdapterAction,
    BidiBluetoothSimulatePreconnectedPeripheralAction,
    BidiEmulationSetGeolocationOverrideAction,
    BidiPermissionsSetPermissionAction,
    BidiSessionSubscribeAction]
