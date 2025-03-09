from typing import Any, List, Mapping

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
    def simulate_adapter(self, context: str, state: str, type_: str) -> Mapping[str, Any]:
        """
        Represents a command `bluetooth.simulateAdapter` specified in
        https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-simulateAdapter-command
        """
        return {
            "context": context,
            "state": state,
            "type": type_
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
