from typing import Any, List, Optional, Mapping, MutableMapping

from ._module import BidiModule, command


class Session(BidiModule):
    @command
    def new(self, capabilities: Mapping[str, Any]) -> Mapping[str, Mapping[str, Any]]:
        return {"capabilities": capabilities}

    @new.result
    def _new(self, result: Mapping[str, Any]) -> Any:
        return result.get("session_id"), result.get("capabilities", {})

    @command
    def subscribe(self,
                  events: List[str],
                  contexts: Optional[List[str]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"events": events}
        if contexts is not None:
            params["contexts"] = contexts
        return params

    @command
    def unsubscribe(self,
                    events: Optional[List[str]] = None,
                    contexts: Optional[List[str]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"events": events if events is not None else []}
        if contexts is not None:
            params["contexts"] = contexts
        return params
