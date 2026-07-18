# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

from __future__ import annotations

import re
from typing import TypeGuard

from cgthings.utils import MakeNativeName
from configuration import Descriptor
from WebIDL import (
    IDLArgument,
    IDLDictionary,
    IDLInterface,
    IDLNullableType,
    IDLObject,
    IDLRecordType,
    IDLSequenceType,
    IDLType,
    IDLTypedefType,
    IDLUnionType,
    IDLWrapperType,
)

# We'll want to insert the indent at the beginnings of lines, but we
# don't want to indent empty lines.  So only indent lines that have a
# non-newline character on them.
lineStartDetector = re.compile("^(?=[^\n#])", re.MULTILINE)


# We'll want to insert the indent at the beginnings of lines, but we
# don't want to indent empty lines.  So only indent lines that have a
# non-newline character on them.
lineStartDetector = re.compile("^(?=[^\n])", re.MULTILINE)


def stripTrailingWhitespace(text: str) -> str:
    tail = "\n" if text.endswith("\n") else ""
    lines = text.splitlines()
    for i in range(len(lines)):
        lines[i] = lines[i].rstrip()
    joined_lines = "\n".join(lines)
    return f"{joined_lines}{tail}"


def dictionaryHasSequenceMember(dictionary: IDLDictionary) -> bool:
    for member in dictionary.members:
        if typeIsSequenceOrHasSequenceMember(member.type):
            return True

    if dictionary.parent:
        # pyrefly: ignore  # bad-argument-type
        return dictionaryHasSequenceMember(dictionary.parent)

    return False


def typeIsSequenceOrHasSequenceMember(type: IDLType) -> bool:
    if type.nullable():
        assert isinstance(type, IDLNullableType)
        type = type.inner
    if type.isSequence():
        return True
    if type.isDictionary():
        # pyrefly: ignore  # missing-attribute
        return dictionaryHasSequenceMember(type.inner)
    if type.isUnion():
        assert isinstance(type, IDLUnionType)
        assert type.flatMemberTypes is not None
        return any(typeIsSequenceOrHasSequenceMember(m.type) for m in type.flatMemberTypes)
    return False


def union_native_type(t: IDLType) -> str:
    name = t.unroll().name
    generic = "::<D>" if containsDomInterface(t) else ""
    return f"GenericUnionTypes::{name}{generic}"


def containsDomInterface(t: IDLObject, logging: bool = False) -> bool:
    if isinstance(t, IDLArgument):
        t = t.type
    if isinstance(t, IDLTypedefType):
        t = t.inner
    while isinstance(t, IDLNullableType) or isinstance(t, IDLWrapperType):
        t = t.inner
    if t.isEnum():
        return False
    if t.isUnion():
        # pyrefly: ignore  # missing-attribute
        return any(map(lambda x: containsDomInterface(x), t.flatMemberTypes))
    if t.isDictionary():
        # pyrefly: ignore  # missing-attribute, bad-argument-type
        return any(map(lambda x: containsDomInterface(x), t.members)) or (t.parent and containsDomInterface(t.parent))
    if isDomInterface(t):
        return True
    assert isinstance(t, IDLType)
    if t.isSequence():
        assert isinstance(t, IDLSequenceType)
        return containsDomInterface(t.inner)
    if t.isRecord():
        assert isinstance(t, IDLRecordType)
        return containsDomInterface(t.inner)
    return False


def isDomInterface(t: IDLObject, logging: bool = False) -> bool:
    while isinstance(t, IDLNullableType) or isinstance(t, IDLWrapperType):
        t = t.inner
    if isinstance(t, IDLInterface):
        return True
    assert isinstance(t, IDLType)
    if t.isCallback() or t.isPromise():
        return True
    return t.isInterface() and (t.isSpiderMonkeyInterface() and not t.isBufferSource())


# Unfortunately, .capitalize() on a string will lowercase things inside the
# string, which we do not want.
def firstCap(string: str) -> str:
    return f"{string[0].upper()}{string[1:]}"


def isIDLType(obj: IDLObject) -> TypeGuard[IDLType]:
    if obj.isType():
        assert isinstance(obj, IDLType)
        return True
    return False


def genericsForType(t: IDLObject) -> tuple[str, str]:
    if containsDomInterface(t):
        return ("<D: DomTypes>", "<D>")
    return ("", "")


def toStringBool(arg: bool) -> str:
    return str(not not arg).lower()


def toBindingNamespace(arg: str) -> str:
    """
    Namespaces are *_Bindings

    actual path is `codegen::Bindings::{toBindingModuleFile(name)}::{toBindingNamespace(name)}`
    """
    return re.sub("((_workers)?$)", "_Binding\\1", MakeNativeName(arg))


def toBindingModuleFile(arg: str) -> str:
    """
    Module files are *Bindings

    actual path is `codegen::Bindings::{toBindingModuleFile(name)}::{toBindingNamespace(name)}`
    """
    return re.sub("((_workers)?$)", "Binding\\1", MakeNativeName(arg))


def toBindingModuleFileFromDescriptor(desc: Descriptor) -> str:
    isSuperModule = desc.maybeGetSuperModule()
    if isSuperModule is not None:
        return toBindingModuleFile(isSuperModule)
    else:
        return toBindingModuleFile(desc.name)


def innerContainerType(type: IDLType) -> IDLType:
    assert type.isSequence() or type.isRecord()
    assert isinstance(type, (IDLSequenceType, IDLRecordType, IDLNullableType))
    return type.inner.inner if type.nullable() else type.inner
