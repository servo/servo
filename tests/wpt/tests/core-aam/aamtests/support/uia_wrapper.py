from __future__ import annotations

from typing import Any, Optional

import comtypes
import comtypes.client

from .api_wrapper import ApiWrapper

comtypes.CoInitialize()

# Load the UI Automation type library. This exposes the IUIAutomation
# interface, the CUIAutomation coclass, and the UIA_* property, control type,
# pattern, and tree scope id constants.
UIA = comtypes.client.GetModule("UIAutomationCore.dll")

# The IUIAutomation entry point, used to obtain the root element, create
# property conditions, and walk the tree.
_automation = comtypes.client.CreateObject(
    UIA.CUIAutomation, interface=UIA.IUIAutomation
)


class UiaConstant(int):
    """An integer that prints its human-readable name in test failures."""

    def __new__(cls, value: int, name: str):
        obj = super().__new__(cls, value)
        obj.name = name
        return obj

    def __repr__(self) -> str:
        # This is what Pytest will show in the error log
        return f"<{self.name}: {int(self)}>"


class _ConstantsProxy:
    """Dynamically routes dot-notation lookups to UIA integer constants."""
    def __init__(self, mapping: dict[int, str]):
        self._id_to_name = mapping
        # Reverse the {id: "Name"} mapping into {"Name": id}
        self._name_to_id = {name: val for val, name in mapping.items()}

    def __getattr__(self, name: str) -> UiaConstant:
        if name in self._name_to_id:
            return UiaConstant(self._name_to_id[name], name)
        raise AttributeError(f"UIA constant '{name}' does not exist.")

    def __dir__(self) -> list[str]:
        # Exposes the names to IDE autocompletion and dir() calls
        return list(self._name_to_id.keys())


# ----  Maps for turning UIA constants into human readable names.

# Generate a mapping of control type id to human readable name, e.g. 50000 -> "Button", 50001 -> "Calendar", etc.
UIA_CONTROL_TYPE_MAP = {
    value: name[len("UIA_"):-len("ControlTypeId")]
    for name, value in vars(UIA).items()
    if name.startswith("UIA_") and name.endswith("ControlTypeId")
}

# Generate a mapping of property id to human readable name, e.g. 30000 -> "AutomationId", 30001 -> "Name", etc.
UIA_PROPERTY_ID_MAP = {
    value: name[len("UIA_"):-len("PropertyId")]
    for name, value in vars(UIA).items()
    if name.startswith("UIA_") and name.endswith("PropertyId")
}

# Generate a mapping of event id to human readable name, e.g. 20000 -> "AutomationFocusChangedEvent", 20001 -> "AutomationPropertyChangedEvent", etc.
UIA_EVENT_ID_MAP = {
    value: name[len("UIA_"):-len("EventId")]
    for name, value in vars(UIA).items()
    if name.startswith("UIA_") and name.endswith("EventId")
}

# Generate a mapping of pattern id to human readable name, e.g. 10000 -> "InvokePattern", 10001 -> "SelectionPattern", etc.
UIA_PATTERN_ID_MAP = {
    value: name[len("UIA_"):-len("PatternId")]
    for name, value in vars(UIA).items()
    if name.startswith("UIA_") and name.endswith("PatternId")
}

# Generate a mapping of landmark type id to human readable name, e.g. 8000 -> "Custom", 80001 -> "Form", etc.
UIA_LANDMARK_TYPE_ID_MAP = {
    value: name[len("UIA_"):-len("LandmarkTypeId")]
    for name, value in vars(UIA).items()
    if name.startswith("UIA_") and name.endswith("LandmarkTypeId")
}

UIA_TEXT_ATTRIBUTE_ID_MAP = {
    value: name[len("UIA_"):-len("AttributeId")]
    for name, value in vars(UIA).items()
    if name.startswith("UIA_") and name.endswith("AttributeId")
}
UIA_TEXT_ATTRIBUTE_NAME_MAP = {name: value for value, name in UIA_TEXT_ATTRIBUTE_ID_MAP.items()}

# Master map used to decode properties back into UiaConstants
_VALUE_MAPS = {
    "ControlType": UIA_CONTROL_TYPE_MAP,
    "PropertyId": UIA_PROPERTY_ID_MAP,
    "EventId": UIA_EVENT_ID_MAP,
    "PatternId": UIA_PATTERN_ID_MAP,
    "LandmarkType": UIA_LANDMARK_TYPE_ID_MAP,
    "TextAttribute": UIA_TEXT_ATTRIBUTE_ID_MAP,
}


# ---- UiaObject wrapper allows mostly for easier reading and writing.

class UiaObject:
    """A single, transparent proxy for Elements, Patterns, and TextRanges."""

    def __init__(self, obj: Any):
        object.__setattr__(self, "_obj", obj)

    def __getattr__(self, name: str) -> Any:
        obj = self._obj

        # 1. Intercept TextAttributes natively
        if name in UIA_TEXT_ATTRIBUTE_NAME_MAP and hasattr(obj, "GetAttributeValue"):
            attr_id = UIA_TEXT_ATTRIBUTE_NAME_MAP[name]
            raw_attr = obj.GetAttributeValue(attr_id)

        # 2. Native
        elif hasattr(obj, name):
            raw_attr = getattr(obj, name)

        else:
            raise AttributeError(
                f"Wrapped UIA object has no attribute '{name}'. "
                f"Available attributes: {dir(obj)}"
            )

        # 4. Handle methods natively
        if callable(raw_attr):
            def method_proxy(*args, **kwargs):
                unwrapped_args = [_unwrap_arg(arg) for arg in args]
                result = raw_attr(*unwrapped_args, **kwargs)

                # --- Decode GetPropertyValue Integers ---
                if name in ("GetCurrentPropertyValue", "GetCachedPropertyValue") and args:
                    prop_name = UIA_PROPERTY_ID_MAP.get(int(args[0]), "")
                    return _decode(prop_name, result)

                # --- Cast Patterns Before Wrapping ---
                if name in ("GetCurrentPattern", "GetCachedPattern") and result and args:
                    pattern_name = UIA_PATTERN_ID_MAP.get(int(args[0]))
                    if pattern_name:
                        interface = getattr(UIA, f"IUIAutomation{pattern_name}Pattern", None)
                        if interface:
                            result = result.QueryInterface(interface)

                return _wrap_object(result)
            return method_proxy

        # 5. Handle properties natively and decode them
        result = _wrap_object(raw_attr)

        base_name = name
        if name.startswith("Current"): base_name = name[7:]
        elif name.startswith("Cached"): base_name = name[6:]

        return _decode(base_name, result)

    def __setattr__(self, name: str, value: Any):
        if name.startswith("_"):
            object.__setattr__(self, name, value)
        else:
            setattr(self._obj, name, value)


# ---- Wrapping & Decoding helpers

def _decode(name: str, result: Any) -> Any:
    """Decodes raw integers into UiaConstants if a map exists for the property."""
    if type(result) is int and name in _VALUE_MAPS:
        str_name = _VALUE_MAPS[name].get(result, "Unknown")
        return UiaConstant(result, str_name)

    if isinstance(result, (list, tuple)) and all(type(x) is int for x in result):
        return [_decode(name, x) for x in result]

    return result

def _unwrap_arg(arg: Any) -> Any:
    """Unwraps proxy objects to hand raw COM pointers back to native UIA methods."""
    return arg._obj if isinstance(arg, UiaObject) else arg

def _wrap_object(result: Any) -> Any:
    """Dynamically wraps raw object UIA COM returns into the universal proxy."""
    if result is None:
        return None

    if hasattr(result, "Length") and hasattr(result, "GetElement"):
        return [_wrap_object(result.GetElement(i)) for i in range(result.Length)]

    type_name = type(result).__name__

    if "Element" in type_name or "TextRange" in type_name or "Pattern" in type_name:
        return UiaObject(result)

    return result


# ---- Main API Wrapper

class UiaWrapper(ApiWrapper[UiaObject]):
    ControlType = _ConstantsProxy(UIA_CONTROL_TYPE_MAP)
    PropertyId = _ConstantsProxy(UIA_PROPERTY_ID_MAP)
    EventId = _ConstantsProxy(UIA_EVENT_ID_MAP)
    PatternId = _ConstantsProxy(UIA_PATTERN_ID_MAP)
    LandmarkType = _ConstantsProxy(UIA_LANDMARK_TYPE_ID_MAP)
    TextAttribute = _ConstantsProxy(UIA_TEXT_ATTRIBUTE_ID_MAP)

    @property
    def api_name(self) -> str:
        return "UIA"

    def find_node(self, dom_id: str, url: str) -> Optional[UiaObject]:
        """
        :param dom_id: The dom id of the node to test.
        :param url: The url of the test.
        """
        if self.test_url != url or not self.document:
            self.test_url = url
            self.document = self._poll_for(
                self._find_tab,
                f"Timeout looking for url: {self.test_url}",
            )

        test_node = self._poll_for(
            lambda: self._find_node_by_id(self.document, dom_id),
            f"Timeout looking for node with id {dom_id} in accessibility API UIA.",
        )

        return test_node

    def _find_browser(self) -> Optional[UiaObject]:
        """Find the UIA element representing the browser's top level window.

        :return: The browser element or None.
        """
        if self.pid and self.pid != 0:
            return self._find_browser_by_pid()
        return self._find_browser_by_name()

    def _find_browser_by_pid(self) -> Optional[UiaElement]:
        """Find the browser window by matching the process id.

        :return: The browser element or None.
        """
        root = _wrap_object(_automation.GetRootElement())
        condition = _automation.CreatePropertyCondition(
            UIA.UIA_ProcessIdPropertyId, self.pid
        )
        return root.FindFirst(UIA.TreeScope_Children, condition)

    def _find_browser_by_name(self) -> Optional[UiaObject]:
        """Find the browser window by matching the product name.

        Used when no pid is available (e.g. servo passes pid 0).

        :return: The browser element or None.
        """
        root = _automation.GetRootElement()
        walker = _automation.ControlViewWalker
        element = _wrap_object(walker.GetFirstChildElement(root))
        while element:
            name = element.CurrentName or ""
            if self.product_name.lower() in name.lower():
                return element
            element = _wrap_object(walker.GetNextSiblingElement(_unwrap_arg(element)))
        return None

    def _find_tab(self) -> Optional[UiaObject]:
        """Find the document with the test url.

        :return: The element representing the test document or None.
        """
        condition = _automation.CreatePropertyCondition(
            UIA.UIA_ControlTypePropertyId, UIA.UIA_DocumentControlTypeId
        )
        wrapped_root = _wrap_object(self.root)

        documents = wrapped_root.FindAll(UIA.TreeScope_Descendants, condition)
        for document in documents:
            if self._document_url(document) == self.test_url:
                return document
        return None

    def _document_url(self, document: UiaObject) -> Optional[str]:
        """Return the url of a document element.

        Browsers expose the document url via the UIA Value property, mirroring
        IAccessible2's accValue on the document.

        :param document: A document control element.
        :return: The url string or None.
        """
        return document.GetCurrentPropertyValue(UIA.UIA_ValueValuePropertyId)

    def _find_node_by_id(
        self, root: UiaObject, dom_id: str
    ) -> Optional[UiaObject]:
        """Find the UIA element with a specified dom_id.

        Browsers expose the DOM id via the UIA AutomationId property.

        :param root: The root node to search from.
        :param dom_id: The DOM id.
        :return: The element or None if not found.
        """
        condition = _automation.CreatePropertyCondition(
            UIA.UIA_AutomationIdPropertyId, dom_id
        )
        return root.FindFirst(UIA.TreeScope_Descendants, condition)
