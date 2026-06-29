# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

from __future__ import annotations

import os

from WebIDL import (
    IDLCallback,
    IDLDictionary,
    IDLObject,
    IDLSequenceType,
    IDLType,
    IDLWrapperType,
)


def MakeNativeName(name: str) -> str:
    return name[0].upper() + name[1:]


def getIdlFileName(object: IDLObject) -> str:
    return os.path.basename(object.location.filename).split(".webidl")[0]


def getModuleFromObject(object: IDLObject) -> str:
    return "crate::codegen::GenericBindings::" + getIdlFileName(object) + "Binding"


def getTypesFromDictionary(dictionary: IDLWrapperType | IDLDictionary) -> list[IDLType]:
    """
    Get all member types for this dictionary
    """
    if isinstance(dictionary, IDLWrapperType):
        dictionary = dictionary.inner
    types = []
    curDict = dictionary
    while curDict:
        assert isinstance(curDict, IDLDictionary)
        types.extend([getUnwrappedType(m.type) for m in curDict.members])
        curDict = curDict.parent
    return types


def getTypesFromCallback(callback: IDLCallback) -> list[IDLType]:
    """
    Get the types this callback depends on: its return type and the
    types of its arguments.
    """
    sig = callback.signatures()[0]
    types = [sig[0]]  # Return type
    types.extend(arg.type for arg in sig[1])  # Arguments
    return types


def getUnwrappedType(type: IDLType) -> IDLType:
    while isinstance(type, IDLSequenceType):
        type = type.inner
    return type
