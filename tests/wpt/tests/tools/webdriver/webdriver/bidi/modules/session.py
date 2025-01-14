from typing import Any, List, Optional, Mapping, MutableMapping

from ._module import BidiModule, command


class Session(BidiModule):
    @command
    def end(self) -> Mapping[str, Any]:
        return {}

    @end.result
    async def _end(self, result: Mapping[str, Any]) -> Any:
        if self.session.transport:
            await self.session.transport.wait_closed()

        return result

    @command
    def new(self, capabilities: Mapping[str, Any]) -> Mapping[str, Mapping[str, Any]]:
        params: MutableMapping[str, Any] = {}
        params["capabilities"] = capabilities
        return params

    @new.result
    def _new(self, result: Mapping[str, Any]) -> Any:
        return result.get("sessionId"), result.get("capabilities", {})

    @command
    def status(self) -> Mapping[str, Any]:
        return {}

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
                    contexts: Optional[List[str]] = None,
                    subscriptions: Optional[List[str]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}
        if contexts is not None:
            params["contexts"] = contexts
        if events is not None:
            params["events"] = events
        if subscriptions is not None:
            params["subscriptions"] = subscriptions
        return params
