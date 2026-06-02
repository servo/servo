# mypy: allow-untyped-defs, allow-untyped-calls

from abc import ABC
import math
from typing import Any, Dict, Union
from .undefined import UNDEFINED

WebElement = None


def do_delayed_imports():
    global WebElement
    from ..client import WebElement

class BidiValue(ABC):
    """Represents the non-primitive values received via BiDi."""
    protocol_value: Dict[str, Any]
    type: str

    def __init__(self, protocol_value: Dict[str, Any]):
        do_delayed_imports()
        assert isinstance(protocol_value, dict)
        assert isinstance(protocol_value["type"], str)
        self.type = protocol_value["type"]
        self.protocol_value = protocol_value

    def to_classic_protocol_value(self) -> Dict[str, Any]:
        """
        Convert the BiDi value to the classic protocol value. Required for
        compatibility of the values sent over BiDi transport with the classic
        actions.
        """
        raise NotImplementedError(
            "No conversion to the classic protocol value is implemented.")


class BidiNode(BidiValue):
    shared_id: str

    def __init__(self, protocol_value: Dict[str, Any]):
        do_delayed_imports()
        super().__init__(protocol_value)
        assert self.type == "node"
        self.shared_id = self.protocol_value["sharedId"]

    def to_classic_protocol_value(self) -> Dict[str, Any]:
        return {WebElement.identifier: self.shared_id}  # type: ignore


class BidiWindow(BidiValue):
    browsing_context: str

    def __init__(self, protocol_value: Dict[str, Any]):
        super().__init__(protocol_value)
        assert self.type == "window"
        self.browsing_context = self.protocol_value["value"]["context"]


def bidi_deserialize(bidi_value: Union[str, int, Dict[str, Any]]) -> Any:
    """
    Deserialize the BiDi primitive values, lists and objects to the Python
    value, keeping non-common data types in BiDi format.
    Note: there can be some ambiguity in the deserialized value.
    Eg `{window: {context: "abc"}}` can represent a window proxy, or the JS
    object `{window: {context: "abc"}}`.
    """
    # script.PrimitiveProtocolValue https://w3c.github.io/webdriver-bidi/#type-script-PrimitiveProtocolValue
    if isinstance(bidi_value, str):
        return bidi_value
    if isinstance(bidi_value, int):
        return bidi_value
    if not isinstance(bidi_value, dict):
        raise ValueError("Unexpected bidi value: %s" % bidi_value)
    if bidi_value["type"] == "undefined":
        return UNDEFINED
    if bidi_value["type"] == "null":
        return None
    if bidi_value["type"] == "string":
        return bidi_value["value"]
    if bidi_value["type"] == "number":
        if bidi_value["value"] == "NaN":
            return math.nan
        if bidi_value["value"] == "-0":
            return -0.0
        if bidi_value["value"] == "Infinity":
            return math.inf
        if bidi_value["value"] == "-Infinity":
            return -math.inf
        if isinstance(bidi_value["value"], int) or isinstance(bidi_value["value"], float):
            return bidi_value["value"]
        raise ValueError("Unexpected bidi value: %s" % bidi_value)
    if bidi_value["type"] == "boolean":
        return bool(bidi_value["value"])
    if bidi_value["type"] == "bigint":
        # Python handles big integers natively.
        return int(bidi_value["value"])
    # script.RemoteValue https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue
    if bidi_value["type"] == "array":
        list_result = []
        for item in bidi_value["value"]:
            list_result.append(bidi_deserialize(item))
        return list_result
    if bidi_value["type"] == "object":
        dict_result = {}
        for item in bidi_value["value"]:
            dict_result[bidi_deserialize(item[0])] = bidi_deserialize(item[1])
        return dict_result
    if bidi_value["type"] == "node":
        return BidiNode(bidi_value)
    if bidi_value["type"] == "window":
        return BidiWindow(bidi_value)
    # TODO: do not raise after verified no regressions in the tests.
    raise ValueError("Unexpected bidi value: %s" % bidi_value)
    # All other types are not deserialized as a generic BidiValue.
    # return BidiValue(bidi_value)
