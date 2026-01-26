import base64
from enum import Enum
from typing import Any, Dict, List, Mapping, MutableMapping, Optional, Union

from ._module import BidiModule, command
from .script import SerializationOptions
from ..undefined import UNDEFINED, Undefined


class ElementOptions(Dict[str, Any]):
    def __init__(self, element: Mapping[str, Any]):
        self["type"] = "element"
        self["element"] = element


class BoxOptions(Dict[str, Any]):
    def __init__(self, x: float, y: float, width: float, height: float):
        self["type"] = "box"
        self["x"] = x
        self["y"] = y
        self["width"] = width
        self["height"] = height


ClipOptions = Union[ElementOptions, BoxOptions]


class OriginOptions(Enum):
    DOCUMENT = "document"
    VIEWPORT = "viewport"


class FormatOptions(Dict[str, Any]):
    def __init__(self, type: str, quality: Optional[float] = None):
        dict.__init__(self, type=type)

        if quality is not None:
            self["quality"] = quality


class BrowsingContext(BidiModule):
    @command
    def activate(self, context: str) -> Mapping[str, Any]:
        return {"context": context}

    @command
    def capture_screenshot(
        self,
        context: str,
        clip: Optional[ClipOptions] = None,
        origin: Optional[OriginOptions] = None,
        format: Optional[FormatOptions] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context}

        if format is not None:
            params["format"] = format
        if clip is not None:
            params["clip"] = clip
        if origin is not None:
            params["origin"] = origin

        return params

    @capture_screenshot.result
    def _capture_screenshot(self, result: Mapping[str, Any]) -> bytes:
        assert isinstance(result["data"], str)

        return base64.b64decode(result["data"])

    @command
    def close(self, context: Optional[str] = None, prompt_unload: Optional[bool] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if context is not None:
            params["context"] = context
        if prompt_unload is not None:
            params["promptUnload"] = prompt_unload

        return params

    @command
    def create(self,
               type_hint: str,
               reference_context: Optional[str] = None,
               background: Optional[bool] = None,
               user_context: Optional[str] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"type": type_hint}

        if reference_context is not None:
            params["referenceContext"] = reference_context

        if background is not None:
            params["background"] = background

        if user_context is not None:
            params["userContext"] = user_context

        return params

    @create.result
    def _create(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["context"], str)

        return result

    @command
    def get_tree(self,
                 max_depth: Optional[int] = None,
                 root: Optional[str] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if max_depth is not None:
            params["maxDepth"] = max_depth
        if root is not None:
            params["root"] = root

        return params

    @get_tree.result
    def _get_tree(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["contexts"], list)

        for context in result["contexts"]:
            self._assert_browsing_context_info(context)

        return result["contexts"]

    @command
    def handle_user_prompt(self,
                           context: str,
                           accept: Optional[bool] = None,
                           user_text: Optional[str] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context}

        if accept is not None:
            params["accept"] = accept
        if user_text is not None:
            params["userText"] = user_text
        return params

    @command
    def locate_nodes(self,
                     context: str,
                     locator: Mapping[str, Any],
                     max_node_count: Optional[int] = None,
                     serialization_options: Optional[SerializationOptions] = None,
                     start_nodes: Optional[List[Mapping[str, Any]]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context, "locator": locator}
        if max_node_count is not None:
            params["maxNodeCount"] = max_node_count
        if serialization_options is not None:
            params["serializationOptions"] = serialization_options
        if start_nodes is not None:
            params["startNodes"] = start_nodes
        return params

    @locate_nodes.result
    def _locate_nodes(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["nodes"], List)

        for node in result["nodes"]:
            self._assert_node_remote_value(node)

        return result

    @command
    def navigate(self,
                 context: str,
                 url: str,
                 wait: Optional[str] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context, "url": url}
        if wait is not None:
            params["wait"] = wait
        return params

    @navigate.result
    def _navigate(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["navigation"], str)
        assert isinstance(result["url"], str)

        return result

    @command
    def print(self,
              context: str,
              background: Optional[bool] = None,
              margin: Optional[Mapping[str, Any]] = None,
              orientation: Optional[str] = None,
              page: Optional[Mapping[str, Any]] = None,
              page_ranges: Optional[List[str]] = None,
              scale: Optional[float] = None,
              shrink_to_fit: Optional[bool] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context}

        if background is not None:
            params["background"] = background
        if margin is not None:
            params["margin"] = margin
        if orientation is not None:
            params["orientation"] = orientation
        if page is not None:
            params["page"] = page
        if page_ranges is not None:
            params["pageRanges"] = page_ranges
        if scale is not None:
            params["scale"] = scale
        if shrink_to_fit is not None:
            params["shrinkToFit"] = shrink_to_fit

        return params

    @print.result
    def _print(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["data"], str)

        return result["data"]

    @command
    def reload(self,
               context: str,
               ignore_cache: Optional[bool] = None,
               wait: Optional[str] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context}
        if ignore_cache is not None:
            params["ignoreCache"] = ignore_cache
        if wait is not None:
            params["wait"] = wait
        return params

    @reload.result
    def _reload(self, result: Mapping[str, Any]) -> Any:
        assert isinstance(result["navigation"], str)
        assert isinstance(result["url"], str)

        return result

    @command
    def set_viewport(self,
                     context: Optional[str] = None,
                     viewport: Union[Optional[Mapping[str, Any]], Undefined] = UNDEFINED,
                     device_pixel_ratio: Union[Optional[float], Undefined] = UNDEFINED,
                     user_contexts: Optional[List[str]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if context is not None:
            params["context"] = context
        if viewport is not UNDEFINED:
            params["viewport"] = viewport
        if device_pixel_ratio is not UNDEFINED:
            params["devicePixelRatio"] = device_pixel_ratio
        if user_contexts is not None:
            params["userContexts"] = user_contexts

        return params

    @command
    def traverse_history(self, context: str, delta: int) -> Mapping[str, Any]:
        return {"context": context, "delta": delta}

    def _assert_browsing_context_info(self, info: Mapping[str, Any]) -> Any:
        assert isinstance(info["clientWindow"], str)
        assert isinstance(info["context"], str)
        assert info["originalOpener"] is None or isinstance(info["originalOpener"], str)
        assert isinstance(info["url"], str)
        assert isinstance(info["userContext"], str)
        if "parent" in info:
            assert info["parent"] is None or isinstance(info["parent"], str)
        assert "children" in info
        if info["children"] is not None:
            for child in info["children"]:
                self._assert_browsing_context_info(child)

    def _assert_node_remote_value(self, node: Mapping[str, Any]) -> Any:
        assert isinstance(node, dict)
        assert isinstance(node["type"], str)
        if "sharedId" in node:
            assert isinstance(node["sharedId"], str)
        if "handle" in node:
            assert isinstance(node["handle"], str)
        if "internalId" in node:
            assert isinstance(node["internalId"], str)
        if "value" in node:
            value = node["value"]
            assert isinstance(value, dict)
            assert isinstance(value["nodeType"], int)
            assert isinstance(value["childNodeCount"], int)
            if "attributes" in value:
                assert isinstance(value["attributes"], dict)
                for k, v in value["attributes"].items():
                    assert isinstance(k, str)
                    assert isinstance(v, str)

            if "children" in value:
                assert isinstance(value["children"], list)
                for child in value["children"]:
                    self._assert_node_remote_value(child)

            if "localName" in value:
                assert isinstance(value["localName"], str)
            if "mode" in value:
                assert value["mode"] in ["open", "closed"]
            if "namespaceURI" in value:
                assert isinstance(value["namespaceURI"], str)
            if "nodeValue" in value:
                assert isinstance(value["nodeValue"], str)
            if "shadowRoot" in value:
                if value["shadowRoot"] is not None:
                    self._assert_node_remote_value(value["shadowRoot"])
