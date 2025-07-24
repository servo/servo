# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# WebIDL type wrapper.

from WebIDL import (
    IDLType,
)
from typing import Protocol, cast


# WARN: This maps the value defined at:
# https://github.com/servo/servo/blob/main/third_party/WebIDL/WebIDL.py#L2525
#
# NOTE: Pyrefly incorrectly infers the type as the class `CostumEnumType` itself,
# instead of our own custom enum implementation. To work around this, we map
# the type definition below. For reference, here is our custom enum implementation:
# https://github.com/servo/servo/blob/main/third_party/WebIDL/WebIDL.py#L52

class IDLTypeTagsProtocol(Protocol):
    int8: int
    uint8: int
    int16: int
    uint16: int
    int32: int
    uint32: int
    int64: int
    uint64: int
    bool: int
    unrestricted_float: int
    float: int
    unrestricted_double: int
    double: int
    any: int
    undefined: int
    domstring: int
    bytestring: int
    usvstring: int
    utf8string: int
    jsstring: int
    object: int
    interface: int
    int8array: int
    uint8array: int
    int16array: int
    uint16array: int
    int32array: int
    uint32array: int
    float32array: int
    float64array: int
    arrayBuffer: int
    arrayBufferView: int
    uint8clampedarray: int
    dictionary: int
    enum: int
    callback: int
    union: int
    sequence: int
    record: int
    promise: int
    observablearray: int

IDLTypeTags = cast(IDLTypeTagsProtocol, IDLType.Tags)
