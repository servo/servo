from typing import Any, Dict, List, Mapping

from ._module import BidiModule, command


class Bluetooth(BidiModule):
    """
    Represents bluetooth automation module specified in
    https://webbluetoothcg.github.io/web-bluetooth/#automated-testing
    """

    @command
    def handle_request_device_prompt(self, context: str, prompt: str, accept: bool, device: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.HandleRequestDevicePrompt` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-handlerequestdeviceprompt-command
        """
        return {
            "context": context,
            "prompt": prompt,
            "accept": accept,
            "device": device,
        }

    @command
    def simulate_adapter(self, context: str, state: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulateAdapter` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulateAdapter-command
        """
        return {
            "context": context,
            "state": state
        }

    @command
    def disable_simulation(self, context: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.disableSimulation` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-disableSimulation-command
        """
        return {
            "context": context,
        }

    @command
    def simulate_preconnected_peripheral(self,
            context: str,
            address: str,
            name: str,
            manufacturer_data: List[Any],
            known_service_uuids: List[str]) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_preconnected_peripheral` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulateconnectedperipheral-command
        """
        return {
            "context": context,
            "address": address,
            "name": name,
            "manufacturerData": manufacturer_data,
            "knownServiceUuids": known_service_uuids
        }

    @command
    def simulate_gatt_connection_response(self,
            context: str,
            address: str,
            code: int) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_gatt_connection_response` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulategattconnectionresponse-command
        """
        return {
            "context": context,
            "address": address,
            "code": code,
        }

    @command
    def simulate_gatt_disconnection(self,
            context: str,
            address: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_gatt_disconnection` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulategattdisconnection-command
        """
        return {
            "context": context,
            "address": address
        }

    @command
    def simulate_service(self,
            context: str,
            address: str,
            uuid: str,
            type: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_service` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulateservice-command
        """
        return {
            "context": context,
            "address": address,
            "uuid": uuid,
            "type": type
        }

    @command
    def simulate_characteristic(self,
            context: str,
            address: str,
            service_uuid: str,
            characteristic_uuid: str,
            characteristic_properties: Dict[str, bool],
            type: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_characteristic` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulatecharacteristic-command
        """
        ret: Dict[str, Any] = {
            "context": context,
            "address": address,
            "serviceUuid": service_uuid,
            "characteristicUuid": characteristic_uuid,
            "type": type
        }
        if characteristic_properties:
            ret["characteristicProperties"] = characteristic_properties
        return ret

    @command
    def simulate_characteristic_response(self,
            context: str,
            address: str,
            service_uuid: str,
            characteristic_uuid: str,
            type: str,
            code: int,
            data: List[int]) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_characteristic_response` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulatecharacteristicresponse-command
        """
        ret: Dict[str, Any] = {
            "context": context,
            "address": address,
            "serviceUuid": service_uuid,
            "characteristicUuid": characteristic_uuid,
            "type": type,
            "code": code
        }
        if data:
            ret["data"] = data
        return ret

    @command
    def simulate_descriptor(self,
            context: str,
            address: str,
            service_uuid: str,
            characteristic_uuid: str,
            descriptor_uuid: str,
            type: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_descriptor` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulatedescriptor-command
        """
        return {
            "context": context,
            "address": address,
            "serviceUuid": service_uuid,
            "characteristicUuid": characteristic_uuid,
            "descriptorUuid": descriptor_uuid,
            "type": type
        }

    @command
    def simulate_descriptor_response(self,
            context: str,
            address: str,
            service_uuid: str,
            characteristic_uuid: str,
            descriptor_uuid: str,
            type: str,
            code: int,
            data: List[int]) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulate_descriptor` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulatedescriptorresponse-command
        """
        ret: Dict[str, Any] = {
            "context": context,
            "address": address,
            "serviceUuid": service_uuid,
            "characteristicUuid": characteristic_uuid,
            "descriptorUuid": descriptor_uuid,
            "type": type,
            "code": code
        }
        if data:
            ret["data"] = data
        return ret
