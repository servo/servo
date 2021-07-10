"""
Tests for `attr.validators`.
"""

from __future__ import absolute_import, division, print_function

import pytest
import zope.interface

import attr

from attr import has
from attr import validators as validator_module
from attr._compat import TYPE
from attr.validators import and_, in_, instance_of, optional, provides

from .utils import simple_attr


class TestInstanceOf(object):
    """
    Tests for `instance_of`.
    """
    def test_success(self):
        """
        Nothing happens if types match.
        """
        v = instance_of(int)
        v(None, simple_attr("test"), 42)

    def test_subclass(self):
        """
        Subclasses are accepted too.
        """
        v = instance_of(int)
        # yep, bools are a subclass of int :(
        v(None, simple_attr("test"), True)

    def test_fail(self):
        """
        Raises `TypeError` on wrong types.
        """
        v = instance_of(int)
        a = simple_attr("test")
        with pytest.raises(TypeError) as e:
            v(None, a, "42")
        assert (
            "'test' must be <{type} 'int'> (got '42' that is a <{type} "
            "'str'>).".format(type=TYPE),
            a, int, "42",

        ) == e.value.args

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = instance_of(int)
        assert (
            "<instance_of validator for type <{type} 'int'>>"
            .format(type=TYPE)
        ) == repr(v)


def always_pass(_, __, ___):
    """
    Toy validator that always passses.
    """


def always_fail(_, __, ___):
    """
    Toy validator that always fails.
    """
    0/0


class TestAnd(object):
    def test_success(self):
        """
        Succeeds if all wrapped validators succeed.
        """
        v = and_(instance_of(int), always_pass)

        v(None, simple_attr("test"), 42)

    def test_fail(self):
        """
        Fails if any wrapped validator fails.
        """
        v = and_(instance_of(int), always_fail)

        with pytest.raises(ZeroDivisionError):
            v(None, simple_attr("test"), 42)

    def test_sugar(self):
        """
        `and_(v1, v2, v3)` and `[v1, v2, v3]` are equivalent.
        """
        @attr.s
        class C(object):
            a1 = attr.ib("a1", validator=and_(
                instance_of(int),
            ))
            a2 = attr.ib("a2", validator=[
                instance_of(int),
            ])

        assert C.__attrs_attrs__[0].validator == C.__attrs_attrs__[1].validator


class IFoo(zope.interface.Interface):
    """
    An interface.
    """
    def f():
        """
        A function called f.
        """


class TestProvides(object):
    """
    Tests for `provides`.
    """
    def test_success(self):
        """
        Nothing happens if value provides requested interface.
        """
        @zope.interface.implementer(IFoo)
        class C(object):
            def f(self):
                pass

        v = provides(IFoo)
        v(None, simple_attr("x"), C())

    def test_fail(self):
        """
        Raises `TypeError` if interfaces isn't provided by value.
        """
        value = object()
        a = simple_attr("x")

        v = provides(IFoo)
        with pytest.raises(TypeError) as e:
            v(None, a, value)
        assert (
            "'x' must provide {interface!r} which {value!r} doesn't."
            .format(interface=IFoo, value=value),
            a, IFoo, value,
        ) == e.value.args

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = provides(IFoo)
        assert (
            "<provides validator for interface {interface!r}>"
            .format(interface=IFoo)
        ) == repr(v)


@pytest.mark.parametrize("validator", [
    instance_of(int),
    [always_pass, instance_of(int)],
])
class TestOptional(object):
    """
    Tests for `optional`.
    """
    def test_success(self, validator):
        """
        Nothing happens if validator succeeds.
        """
        v = optional(validator)
        v(None, simple_attr("test"), 42)

    def test_success_with_none(self, validator):
        """
        Nothing happens if None.
        """
        v = optional(validator)
        v(None, simple_attr("test"), None)

    def test_fail(self, validator):
        """
        Raises `TypeError` on wrong types.
        """
        v = optional(validator)
        a = simple_attr("test")
        with pytest.raises(TypeError) as e:
            v(None, a, "42")
        assert (
            "'test' must be <{type} 'int'> (got '42' that is a <{type} "
            "'str'>).".format(type=TYPE),
            a, int, "42",

        ) == e.value.args

    def test_repr(self, validator):
        """
        Returned validator has a useful `__repr__`.
        """
        v = optional(validator)

        if isinstance(validator, list):
            assert (
                ("<optional validator for _AndValidator(_validators=[{func}, "
                 "<instance_of validator for type <{type} 'int'>>]) or None>")
                .format(func=repr(always_pass), type=TYPE)
            ) == repr(v)
        else:
            assert (
                ("<optional validator for <instance_of validator for type "
                 "<{type} 'int'>> or None>")
                .format(type=TYPE)
            ) == repr(v)


class TestIn_(object):
    """
    Tests for `in_`.
    """
    def test_success_with_value(self):
        """
        If the value is in our options, nothing happens.
        """
        v = in_([1, 2, 3])
        a = simple_attr("test")
        v(1, a, 3)

    def test_fail(self):
        """
        Raise ValueError if the value is outside our options.
        """
        v = in_([1, 2, 3])
        a = simple_attr("test")
        with pytest.raises(ValueError) as e:
            v(None, a, None)
        assert (
            "'test' must be in [1, 2, 3] (got None)",
        ) == e.value.args

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = in_([3, 4, 5])
        assert(
            ("<in_ validator with options [3, 4, 5]>")
        ) == repr(v)


def test_hashability():
    """
    Validator classes are hashable.
    """
    for obj_name in dir(validator_module):
        obj = getattr(validator_module, obj_name)
        if not has(obj):
            continue
        hash_func = getattr(obj, '__hash__', None)
        assert hash_func is not None
        assert hash_func is not object.__hash__
