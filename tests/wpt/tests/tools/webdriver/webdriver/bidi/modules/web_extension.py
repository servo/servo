from typing import Any, Mapping, MutableMapping

from ._module import BidiModule, command

class WebExtension(BidiModule):
    @command
    def install(
        self,
        extension_data: Mapping[str, Any],
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"extensionData": extension_data}
        return params

    @install.result
    def _install(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["extension"], str)
        return result["extension"]

    @command
    def uninstall(self, extension: str) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"extension": extension}
        return params
