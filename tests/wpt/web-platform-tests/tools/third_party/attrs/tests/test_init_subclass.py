# SPDX-License-Identifier: MIT

"""
Tests for `__init_subclass__` related tests.

Python 3.6+ only.
"""

import pytest

import attr


@pytest.mark.parametrize("slots", [True, False])
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
