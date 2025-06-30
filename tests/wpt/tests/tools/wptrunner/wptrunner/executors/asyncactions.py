# mypy: allow-untyped-defs

from typing import Any, Mapping
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

class BidiBluetoothAction:
    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def __call__(self, payload):
        if "context" not in payload:
            raise ValueError("Missing required parameter: context")

        context = get_browsing_context_id(payload["context"])
        if isinstance(context, str):
            pass
        elif isinstance(context, webdriver.bidi.protocol.BidiWindow):
            # Context can be a serialized WindowProxy.
            context = context.browsing_context
        else:
            raise ValueError("Unexpected context type: %s" % context)
        return await self.execute(context, payload)

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        raise NotImplementedError

class BidiBluetoothHandleRequestDevicePrompt(BidiBluetoothAction):
    name = "bidi.bluetooth.handle_request_device_prompt"

    def __init__(self, logger, protocol):
        do_delayed_imports()
        self.logger = logger
        self.protocol = protocol

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        prompt = payload["prompt"]
        accept = payload["accept"]
        device = payload["device"]
        return await self.protocol.bidi_bluetooth.handle_request_device_prompt(context, prompt, accept, device)

class BidiBluetoothSimulateAdapterAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_adapter"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        state = payload["state"]
        return await self.protocol.bidi_bluetooth.simulate_adapter(context,
                                                                   state)

class BidiBluetoothDisableSimulationAction(BidiBluetoothAction):
    name = "bidi.bluetooth.disable_simulation"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        return await self.protocol.bidi_bluetooth.disable_simulation(context)

class BidiBluetoothSimulatePreconnectedPeripheralAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_preconnected_peripheral"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        name = payload["name"]
        manufacturer_data = payload["manufacturerData"]
        known_service_uuids = payload["knownServiceUuids"]
        return await self.protocol.bidi_bluetooth.simulate_preconnected_peripheral(
            context, address, name, manufacturer_data, known_service_uuids)

class BidiBluetoothSimulateGattConnectionResponseAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_gatt_connection_response"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        code = payload["code"]
        return await self.protocol.bidi_bluetooth.simulate_gatt_connection_response(
            context, address, code)

class BidiBluetoothSimulateGattDisconnectionAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_gatt_disconnection"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        return await self.protocol.bidi_bluetooth.simulate_gatt_disconnection(
            context, address)

class BidiBluetoothSimulateServiceAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_service"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        uuid = payload["uuid"]
        type = payload["type"]
        return await self.protocol.bidi_bluetooth.simulate_service(
            context, address, uuid, type)

class BidiBluetoothSimulateCharacteristicAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_characteristic"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        service_uuid = payload["serviceUuid"]
        characteristic_uuid = payload["characteristicUuid"]
        characteristic_properties = payload["characteristicProperties"]
        type = payload["type"]
        return await self.protocol.bidi_bluetooth.simulate_characteristic(
            context, address, service_uuid, characteristic_uuid, characteristic_properties, type)

class BidiBluetoothSimulateCharacteristicResponseAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_characteristic_response"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        service_uuid = payload["serviceUuid"]
        characteristic_uuid = payload["characteristicUuid"]
        type = payload["type"]
        code = payload["code"]
        data = payload["data"]
        return await self.protocol.bidi_bluetooth.simulate_characteristic_response(
            context, address, service_uuid, characteristic_uuid, type, code, data)

class BidiBluetoothSimulateDescriptorAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_descriptor"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        service_uuid = payload["serviceUuid"]
        characteristic_uuid = payload["characteristicUuid"]
        descriptor_uuid = payload["descriptorUuid"]
        type = payload["type"]
        return await self.protocol.bidi_bluetooth.simulate_descriptor(
            context, address, service_uuid, characteristic_uuid, descriptor_uuid, type)

class BidiBluetoothSimulateDescriptorResponseAction(BidiBluetoothAction):
    name = "bidi.bluetooth.simulate_descriptor_response"

    async def execute(self, context: str, payload: Mapping[str, Any]) -> Any:
        address = payload["address"]
        service_uuid = payload["serviceUuid"]
        characteristic_uuid = payload["characteristicUuid"]
        descriptor_uuid = payload["descriptorUuid"]
        type = payload["type"]
        code = payload["code"]
        data = payload["data"]
        return await self.protocol.bidi_bluetooth.simulate_descriptor_response(
            context, address, service_uuid, characteristic_uuid, descriptor_uuid, type, code, data)

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
            None if error is None else webdriver.bidi.undefined.UNDEFINED)

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
    BidiBluetoothDisableSimulationAction,
    BidiBluetoothSimulatePreconnectedPeripheralAction,
    BidiBluetoothSimulateGattConnectionResponseAction,
    BidiBluetoothSimulateGattDisconnectionAction,
    BidiBluetoothSimulateServiceAction,
    BidiBluetoothSimulateCharacteristicAction,
    BidiBluetoothSimulateCharacteristicResponseAction,
    BidiBluetoothSimulateDescriptorAction,
    BidiBluetoothSimulateDescriptorResponseAction,
    BidiEmulationSetGeolocationOverrideAction,
    BidiPermissionsSetPermissionAction,
    BidiSessionSubscribeAction]
