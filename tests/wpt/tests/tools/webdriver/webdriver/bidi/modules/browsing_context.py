import base64
from typing import Any, Dict, List, Mapping, MutableMapping, Optional, Union

from ._module import BidiModule, command


class ElementOptions(Dict[str, Any]):
    def __init__(
        self, element: Mapping[str, Any], scroll_into_view: Optional[bool] = None
    ):
        self["type"] = "element"
        self["element"] = element

        if scroll_into_view is not None:
            self["scrollIntoView"] = scroll_into_view


class ViewportOptions(Dict[str, Any]):
    def __init__(self, x: float, y: float, width: float, height: float):
        self["type"] = "viewport"
        self["x"] = x
        self["y"] = y
        self["width"] = width
        self["height"] = height


ClipOptions = Union[ElementOptions, ViewportOptions]


class BrowsingContext(BidiModule):
    @command
    def activate(self, context: str) -> Mapping[str, Any]:
        return {"context": context}

    @command
    def capture_screenshot(
        self, context: str, clip: Optional[ClipOptions] = None
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context}

        if clip is not None:
            params["clip"] = clip

        return params

    @capture_screenshot.result
    def _capture_screenshot(self, result: Mapping[str, Any]) -> bytes:
        assert result["data"] is not None
        return base64.b64decode(result["data"])

    @command
    def close(self, context: Optional[str] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if context is not None:
            params["context"] = context

        return params

    @command
    def create(self,
               type_hint: str,
               reference_context: Optional[str] = None,
               background: Optional[bool] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"type": type_hint}

        if reference_context is not None:
            params["referenceContext"] = reference_context

        if background is not None:
            params["background"] = background

        return params

    @create.result
    def _create(self, result: Mapping[str, Any]) -> Any:
        assert result["context"] is not None

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
        assert result["contexts"] is not None
        assert isinstance(result["contexts"], list)

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
        if result["navigation"] is not None:
            assert isinstance(result["navigation"], str)

        assert result["url"] is not None
        assert isinstance(result["url"], str)

        return result

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
        assert result["data"] is not None
        return result["data"]

    @command
    def set_viewport(self, context: str, viewport: Optional[Mapping[str, Any]] = None) -> Mapping[str, Any]:
        return {"context": context, "viewport": viewport}
