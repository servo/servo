# SPDX-License-Identifier: MIT

import abc
import inspect

import pytest

import attrs

from attr._compat import PY310, PY_3_12_PLUS


@pytest.mark.skipif(not PY310, reason="abc.update_abstractmethods is 3.10+")
class TestUpdateAbstractMethods:
    def test_abc_implementation(self, slots):
        """
        If an attrs class implements an abstract method, it stops being
        abstract.
        """

        class Ordered(abc.ABC):
            @abc.abstractmethod
            def __lt__(self, other):
                pass

            @abc.abstractmethod
            def __le__(self, other):
                pass

        @attrs.define(order=True, slots=slots)
        class Concrete(Ordered):
            x: int

        assert not inspect.isabstract(Concrete)
        assert Concrete(2) > Concrete(1)

    def test_remain_abstract(self, slots):
        """
        If an attrs class inherits from an abstract class but doesn't implement
        abstract methods, it remains abstract.
        """

        class A(abc.ABC):
            @abc.abstractmethod
            def foo(self):
                pass

        @attrs.define(slots=slots)
        class StillAbstract(A):
            pass

        assert inspect.isabstract(StillAbstract)
        expected_exception_message = (
            "^Can't instantiate abstract class StillAbstract without an "
            "implementation for abstract method 'foo'$"
            if PY_3_12_PLUS
            else "class StillAbstract with abstract method foo"
        )
        with pytest.raises(TypeError, match=expected_exception_message):
            StillAbstract()
