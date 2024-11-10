from typing import Any, Mapping

from ._module import BidiModule, command


class Bluetooth(BidiModule):
    """
    Represents bluetooth automation module specified in
    https://webbluetoothcg.github.io/web-bluetooth/#automated-testing
    """

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
