# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

from __future__ import annotations

import re
from abc import abstractmethod

from cgthings.helpers import lineStartDetector, stripTrailingWhitespace
from WebIDL import (
    IDLType,
)


class CGThing:
    """
    Abstract base class for things that spit out code.
    """

    def __init__(self) -> None:
        pass  # Nothing for now

    @abstractmethod
    def define(self) -> str:
        """Produce code for a Rust file."""
        raise NotImplementedError


class CGRecord(CGThing):
    """
    CGThing that wraps value CGThing in record with key type equal to keyType parameter
    """

    def __init__(self, keyType: IDLType, value: CGThing) -> None:
        CGThing.__init__(self)
        assert keyType.isString()
        self.keyType = keyType
        self.value = value

    def define(self) -> str:
        if self.keyType.isByteString():
            keyDef = "ByteString"
        elif self.keyType.isDOMString():
            keyDef = "DOMString"
        elif self.keyType.isUSVString():
            keyDef = "USVString"
        else:
            assert False

        defn = f"{keyDef}, {self.value.define()}"
        return f"Record<{defn}>"


class CGIndenter(CGThing):
    """
    A class that takes another CGThing and generates code that indents that
    CGThing by some number of spaces.  The default indent is two spaces.
    """

    def __init__(self, child: CGThing, indentLevel: int = 4) -> None:
        CGThing.__init__(self)
        self.child = child
        self.indent = " " * indentLevel

    def define(self) -> str:
        defn = self.child.define()
        if defn != "":
            return re.sub(lineStartDetector, self.indent, defn)
        else:
            return defn


class CGWrapper(CGThing):
    """
    Generic CGThing that wraps other CGThings with pre and post text.
    """

    child: CGThing
    pre: str
    post: str
    reindent: bool

    def __init__(self, child: CGThing, pre: str = "", post: str = "", reindent: bool = False) -> None:
        CGThing.__init__(self)
        self.child = child
        self.pre = pre
        self.post = post
        self.reindent = reindent

    def define(self) -> str:
        defn = self.child.define()
        if self.reindent:
            # We don't use lineStartDetector because we don't want to
            # insert whitespace at the beginning of our _first_ line.
            defn = stripTrailingWhitespace(defn.replace("\n", f"\n{' ' * len(self.pre)}"))
        return f"{self.pre}{defn}{self.post}"


class CGGeneric(CGThing):
    """
    A class that spits out a fixed string into the codegen.  Can spit out a
    separate string for the declaration too.
    """

    text: str

    def __init__(self, text: str) -> None:
        self.text = text

    def define(self) -> str:
        return self.text
