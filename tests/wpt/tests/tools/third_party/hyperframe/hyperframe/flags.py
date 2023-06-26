# -*- coding: utf-8 -*-
"""
hyperframe/flags
~~~~~~~~~~~~~~~~

Defines basic Flag and Flags data structures.
"""
import collections

try:
    from collections.abc import MutableSet
except ImportError:  # pragma: no cover
    # Python 2.7 compatibility
    from collections import MutableSet

Flag = collections.namedtuple("Flag", ["name", "bit"])


class Flags(MutableSet):
    """
    A simple MutableSet implementation that will only accept known flags as
    elements.

    Will behave like a regular set(), except that a ValueError will be thrown
    when .add()ing unexpected flags.
    """
    def __init__(self, defined_flags):
        self._valid_flags = set(flag.name for flag in defined_flags)
        self._flags = set()

    def __contains__(self, x):
        return self._flags.__contains__(x)

    def __iter__(self):
        return self._flags.__iter__()

    def __len__(self):
        return self._flags.__len__()

    def discard(self, value):
        return self._flags.discard(value)

    def add(self, value):
        if value not in self._valid_flags:
            raise ValueError(
                "Unexpected flag: {}. Valid flags are: {}".format(
                    value, self._valid_flags
                )
            )
        return self._flags.add(value)
