# SPDX-License-Identifier: MIT

"""
Tests for `__init_subclass__` related functionality.
"""

import attr


def test_init_subclass_vanilla(slots):
    """
    `super().__init_subclass__` can be used if the subclass is not an attrs
    class both with dict and slotted classes.
    """

    @attr.s(slots=slots)
    class Base:
        def __init_subclass__(cls, param, **kw):
            super().__init_subclass__(**kw)
            cls.param = param

    class Vanilla(Base, param="foo"):
        pass

    assert "foo" == Vanilla().param


def test_init_subclass_attrs():
    """
    `__init_subclass__` works with attrs classes as long as slots=False.
    """

    @attr.s(slots=False)
    class Base:
        def __init_subclass__(cls, param, **kw):
            super().__init_subclass__(**kw)
            cls.param = param

    @attr.s
    class Attrs(Base, param="foo"):
        pass

    assert "foo" == Attrs().param


def test_init_subclass_slots_workaround():
    """
    `__init_subclass__` works with modern APIs if care is taken around classes
    existing twice.
    """
    subs = {}

    @attr.define
    class Base:
        def __init_subclass__(cls):
            subs[cls.__qualname__] = cls

    @attr.define
    class Sub1(Base):
        x: int

    @attr.define
    class Sub2(Base):
        y: int

    assert (Sub1, Sub2) == tuple(subs.values())
