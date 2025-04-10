from typing import Any, Mapping, MutableMapping, Optional

from ._module import BidiModule, command

class WebExtension(BidiModule):
    @command
    def install(
        self,
        extension_data: Mapping[str, Any],
        **kwargs: Mapping[str, Any]
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"extensionData": extension_data}

        # Allow vendor specific parameters
        params.update(**kwargs)

        return params

    @install.result
    def _install(self, result: Mapping[str, Any]) -> Optional[str]:
        return result.get("extension")

    @command
    def uninstall(self, extension: str) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"extension": extension}
        return params
