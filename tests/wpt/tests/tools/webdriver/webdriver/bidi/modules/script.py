from enum import Enum
from typing import Any, Dict, List, Mapping, MutableMapping, Optional, Union

from ..error import UnknownErrorException
from ._module import BidiModule, command


class ScriptEvaluateResultException(Exception):
    def __init__(self, result: Mapping[str, Any]):
        super().__init__()

        self.result = result

        details = result.get("exceptionDetails", {})
        self.column_number = details.get("columnNumber")
        self.exception = details.get("exception")
        self.line_number = details.get("lineNumber")
        self.stacktrace = self.process_stacktrace(details.get("stackTrace", {}))
        self.text = details.get("text")

    def process_stacktrace(self, stacktrace: Mapping[str, Any]) -> str:
        stack = ""
        for frame in stacktrace.get("callFrames", []):
            data = frame.get("functionName") or "eval code"
            if "url" in frame:
                data += f"@{frame['url']}"
            data += f":{frame.get('lineNumber', 0)}:{frame.get('columnNumber', 0)}"
            stack += data + "\n"

        return stack

    def __repr__(self) -> str:
        """Return the object representation in string format."""
        return f"<{self.__class__.__name__}(), {self.text})>"

    def __str__(self) -> str:
        """Return the string representation of the object."""
        message: str = self.text

        if self.stacktrace:
            message += f"\n\nStacktrace:\n\n{self.stacktrace}"

        return message


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


class SerializationOptions(Dict[str, Any]):
    def __init__(
            self,
            max_dom_depth: Optional[int] = None,
            max_object_depth: Optional[int] = None,
            include_shadow_tree: Optional[str] = None
    ):
        if max_dom_depth is not None:
            self["maxDomDepth"] = max_dom_depth
        if max_object_depth is not None:
            self["maxObjectDepth"] = max_object_depth
        if include_shadow_tree is not None:
            self["includeShadowTree"] = include_shadow_tree


class Script(BidiModule):
    @command
    def add_preload_script(
        self,
        function_declaration: str,
        arguments: Optional[List[Mapping[str, Any]]] = None,
        contexts: Optional[List[str]] = None,
        sandbox: Optional[str] = None
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "functionDeclaration": function_declaration
        }

        if arguments is not None:
            params["arguments"] = arguments
        if contexts is not None:
            params["contexts"] = contexts
        if sandbox is not None:
            params["sandbox"] = sandbox

        return params

    @add_preload_script.result
    def _add_preload_script(self, result: Mapping[str, Any]) -> Any:
        assert "script" in result

        return result["script"]

    @command
    def call_function(
        self,
        function_declaration: str,
        await_promise: bool,
        target: Target,
        arguments: Optional[List[Mapping[str, Any]]] = None,
        this: Optional[Mapping[str, Any]] = None,
        result_ownership: Optional[OwnershipModel] = None,
        serialization_options: Optional[SerializationOptions] = None,
        user_activation: Optional[bool] = None
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
        if serialization_options is not None:
            params["serializationOptions"] = serialization_options
        if user_activation is not None:
            params["userActivation"] = user_activation
        return params

    @call_function.result
    def _call_function(self, result: Mapping[str, Any]) -> Any:
        assert "type" in result

        if result["type"] == "success":
            return result["result"]
        elif result["type"] == "exception":
            raise ScriptEvaluateResultException(result)
        else:
            raise UnknownErrorException(f"""Invalid type '{result["type"]}' in response""")

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
        serialization_options: Optional[SerializationOptions] = None,
        user_activation: Optional[bool] = None
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "expression": expression,
            "target": target,
            "awaitPromise": await_promise,
        }

        if result_ownership is not None:
            params["resultOwnership"] = result_ownership
        if serialization_options is not None:
            params["serializationOptions"] = serialization_options
        if user_activation is not None:
            params["userActivation"] = user_activation
        return params

    @evaluate.result
    def _evaluate(self, result: Mapping[str, Any]) -> Any:
        assert "type" in result

        if result["type"] == "success":
            return result["result"]
        elif result["type"] == "exception":
            raise ScriptEvaluateResultException(result)
        else:
            raise UnknownErrorException(f"""Invalid type '{result["type"]}' in response""")

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

    @command
    def remove_preload_script(self, script: str) -> Any:
        params: MutableMapping[str, Any] = {
            "script": script
        }

        return params
