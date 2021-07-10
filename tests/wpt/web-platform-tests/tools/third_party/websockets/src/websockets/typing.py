from typing import List, NewType, Optional, Tuple, Union


__all__ = ["Data", "Origin", "ExtensionHeader", "ExtensionParameter", "Subprotocol"]

Data = Union[str, bytes]

Data__doc__ = """
Types supported in a WebSocket message:

- :class:`str` for text messages
- :class:`bytes` for binary messages

"""
# Remove try / except when dropping support for Python < 3.7
try:
    Data.__doc__ = Data__doc__  # type: ignore
except AttributeError:  # pragma: no cover
    pass


Origin = NewType("Origin", str)
Origin.__doc__ = """Value of a Origin header"""


ExtensionName = NewType("ExtensionName", str)
ExtensionName.__doc__ = """Name of a WebSocket extension"""


ExtensionParameter = Tuple[str, Optional[str]]

ExtensionParameter__doc__ = """Parameter of a WebSocket extension"""
try:
    ExtensionParameter.__doc__ = ExtensionParameter__doc__  # type: ignore
except AttributeError:  # pragma: no cover
    pass


ExtensionHeader = Tuple[ExtensionName, List[ExtensionParameter]]

ExtensionHeader__doc__ = """Item parsed in a Sec-WebSocket-Extensions header"""
try:
    ExtensionHeader.__doc__ = ExtensionHeader__doc__  # type: ignore
except AttributeError:  # pragma: no cover
    pass


Subprotocol = NewType("Subprotocol", str)
Subprotocol.__doc__ = """Items parsed in a Sec-WebSocket-Protocol header"""
