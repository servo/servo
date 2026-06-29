# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

from __future__ import annotations

import functools
import operator
import os
import re
import string
import textwrap
from abc import abstractmethod
from collections import defaultdict
from collections.abc import Callable, Generator, Iterable, Iterator
from enum import IntEnum
from itertools import groupby
from re import Match
from typing import Any, Generic, Optional, TypeGuard, TypeVar, cast

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
