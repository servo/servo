from enum import Enum
from typing import Any, Optional, Mapping, List, MutableMapping, Union, Dict

from ._module import BidiModule, command


class ScriptEvaluateResultException(Exception):
    def __init__(self, result: Mapping[str, Any]):
        self.result = result
        super().__init__("Script execution failed.")


class OwnershipModel(Enum):
    NONE = "none"
    ROOT = "root"


class RealmTypes(Enum):
    AUDIO_WORKLET = "audio-worklet"
    DEDICATED_WORKER = "dedicated-worker"
    PAINT_WORKLET = "paint-worklet"
    SERVICE_WORKER = "service-worker"
    SHARED_WORKER = "shared-worker"
    WINDOW = "window"
    WORKER = "worker"
    WORKLET = "worklet"


class RealmTarget(Dict[str, Any]):
    def __init__(self, realm: str):
        dict.__init__(self, realm=realm)


class ContextTarget(Dict[str, Any]):
    def __init__(self, context: str, sandbox: Optional[str] = None):
        if sandbox is None:
            dict.__init__(self, context=context)
        else:
            dict.__init__(self, context=context, sandbox=sandbox)


Target = Union[RealmTarget, ContextTarget]


class Script(BidiModule):
    @command
    def call_function(
        self,
        function_declaration: str,
        await_promise: bool,
        target: Target,
        arguments: Optional[List[Mapping[str, Any]]] = None,
        this: Optional[Mapping[str, Any]] = None,
        result_ownership: Optional[OwnershipModel] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "functionDeclaration": function_declaration,
            "target": target,
            "awaitPromise": await_promise,
        }

        if arguments is not None:
            params["arguments"] = arguments
        if this is not None:
            params["this"] = this
        if result_ownership is not None:
            params["resultOwnership"] = result_ownership
        return params

    @call_function.result
    def _call_function(self, result: Mapping[str, Any]) -> Any:
        if "result" not in result:
            raise ScriptEvaluateResultException(result)
        return result["result"]

    @command
    def disown(self, handles: List[str], target: Target) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"handles": handles, "target": target}
        return params

    @command
    def evaluate(
        self,
        expression: str,
        target: Target,
        await_promise: bool,
        result_ownership: Optional[OwnershipModel] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "expression": expression,
            "target": target,
            "awaitPromise": await_promise,
        }

        if result_ownership is not None:
            params["resultOwnership"] = result_ownership
        return params

    @evaluate.result
    def _evaluate(self, result: Mapping[str, Any]) -> Any:
        if "result" not in result:
            raise ScriptEvaluateResultException(result)
        return result["result"]

    @command
    def get_realms(
        self,
        context: Optional[str] = None,
        type: Optional[RealmTypes] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if context is not None:
            params["context"] = context
        if type is not None:
            params["type"] = type

        return params

    @get_realms.result
    def _get_realms(self, result: Mapping[str, Any]) -> Any:
        assert result["realms"] is not None
        assert isinstance(result["realms"], list)

        return result["realms"]
