"""
Tests for dunder methods from `attrib._make`.
"""

from __future__ import absolute_import, division, print_function

import copy

import pytest

from hypothesis import given
from hypothesis.strategies import booleans

import attr

from attr._make import (
    NOTHING, Factory, _add_init, _add_repr, _Nothing, fields, make_class
)
from attr.validators import instance_of

from .utils import simple_attr, simple_class


CmpC = simple_class(cmp=True)
CmpCSlots = simple_class(cmp=True, slots=True)
ReprC = simple_class(repr=True)
ReprCSlots = simple_class(repr=True, slots=True)

# HashC is hashable by explicit definition while HashCSlots is hashable
# implicitly.
HashC = simple_class(hash=True)
HashCSlots = simple_class(hash=None, cmp=True, frozen=True, slots=True)


class InitC(object):
    __attrs_attrs__ = [simple_attr("a"), simple_attr("b")]


InitC = _add_init(InitC, False)


class TestAddCmp(object):
    """
    Tests for `_add_cmp`.
    """
    @given(booleans())
    def test_cmp(self, slots):
        """
        If `cmp` is False, ignore that attribute.
        """
        C = make_class("C", {
            "a": attr.ib(cmp=False),
            "b": attr.ib()
        }, slots=slots)

        assert C(1, 2) == C(2, 2)

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_equal(self, cls):
        """
        Equal objects are detected as equal.
        """
        assert cls(1, 2) == cls(1, 2)
        assert not (cls(1, 2) != cls(1, 2))

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_unequal_same_class(self, cls):
        """
        Unequal objects of correct type are detected as unequal.
        """
        assert cls(1, 2) != cls(2, 1)
        assert not (cls(1, 2) == cls(2, 1))

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_unequal_different_class(self, cls):
        """
        Unequal objects of different type are detected even if their attributes
        match.
        """
        class NotCmpC(object):
            a = 1
            b = 2
        assert cls(1, 2) != NotCmpC()
        assert not (cls(1, 2) == NotCmpC())

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_lt(self, cls):
        """
        __lt__ compares objects as tuples of attribute values.
        """
        for a, b in [
            ((1, 2),  (2, 1)),
            ((1, 2),  (1, 3)),
            (("a", "b"), ("b", "a")),
        ]:
            assert cls(*a) < cls(*b)

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_lt_unordable(self, cls):
        """
        __lt__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__lt__(42))

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_le(self, cls):
        """
        __le__ compares objects as tuples of attribute values.
        """
        for a, b in [
            ((1, 2),  (2, 1)),
            ((1, 2),  (1, 3)),
            ((1, 1),  (1, 1)),
            (("a", "b"), ("b", "a")),
            (("a", "b"), ("a", "b")),
        ]:
            assert cls(*a) <= cls(*b)

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_le_unordable(self, cls):
        """
        __le__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__le__(42))

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_gt(self, cls):
        """
        __gt__ compares objects as tuples of attribute values.
        """
        for a, b in [
            ((2, 1), (1, 2)),
            ((1, 3), (1, 2)),
            (("b", "a"), ("a", "b")),
        ]:
            assert cls(*a) > cls(*b)

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_gt_unordable(self, cls):
        """
        __gt__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__gt__(42))

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_ge(self, cls):
        """
        __ge__ compares objects as tuples of attribute values.
        """
        for a, b in [
            ((2, 1), (1, 2)),
            ((1, 3), (1, 2)),
            ((1, 1), (1, 1)),
            (("b", "a"), ("a", "b")),
            (("a", "b"), ("a", "b")),
        ]:
            assert cls(*a) >= cls(*b)

    @pytest.mark.parametrize("cls", [CmpC, CmpCSlots])
    def test_ge_unordable(self, cls):
        """
        __ge__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__ge__(42))


class TestAddRepr(object):
    """
    Tests for `_add_repr`.
    """
    @pytest.mark.parametrize("slots", [True, False])
    def test_repr(self, slots):
        """
        If `repr` is False, ignore that attribute.
        """
        C = make_class("C", {
            "a": attr.ib(repr=False),
            "b": attr.ib()
        }, slots=slots)

        assert "C(b=2)" == repr(C(1, 2))

    @pytest.mark.parametrize("cls", [ReprC, ReprCSlots])
    def test_repr_works(self, cls):
        """
        repr returns a sensible value.
        """
        assert "C(a=1, b=2)" == repr(cls(1, 2))

    def test_infinite_recursion(self):
        """
        In the presence of a cyclic graph, repr will emit an ellipsis and not
        raise an exception.
        """
        @attr.s
        class Cycle(object):
            value = attr.ib(default=7)
            cycle = attr.ib(default=None)

        cycle = Cycle()
        cycle.cycle = cycle
        assert "Cycle(value=7, cycle=...)" == repr(cycle)

    def test_underscores(self):
        """
        repr does not strip underscores.
        """
        class C(object):
            __attrs_attrs__ = [simple_attr("_x")]

        C = _add_repr(C)
        i = C()
        i._x = 42

        assert "C(_x=42)" == repr(i)

    def test_repr_uninitialized_member(self):
        """
        repr signals unset attributes
        """
        C = make_class("C", {
            "a": attr.ib(init=False),
        })

        assert "C(a=NOTHING)" == repr(C())

    @given(add_str=booleans(), slots=booleans())
    def test_str(self, add_str, slots):
        """
        If str is True, it returns the same as repr.

        This only makes sense when subclassing a class with an poor __str__
        (like Exceptions).
        """
        @attr.s(str=add_str, slots=slots)
        class Error(Exception):
            x = attr.ib()

        e = Error(42)

        assert (str(e) == repr(e)) is add_str

    def test_str_no_repr(self):
        """
        Raises a ValueError if repr=False and str=True.
        """
        with pytest.raises(ValueError) as e:
            simple_class(repr=False, str=True)

        assert (
            "__str__ can only be generated if a __repr__ exists."
        ) == e.value.args[0]


class TestAddHash(object):
    """
    Tests for `_add_hash`.
    """
    def test_enforces_type(self):
        """
        The `hash` argument to both attrs and attrib must be None, True, or
        False.
        """
        exc_args = ("Invalid value for hash.  Must be True, False, or None.",)

        with pytest.raises(TypeError) as e:
            make_class("C", {}, hash=1),

        assert exc_args == e.value.args

        with pytest.raises(TypeError) as e:
            make_class("C", {"a": attr.ib(hash=1)}),

        assert exc_args == e.value.args

    @given(booleans())
    def test_hash_attribute(self, slots):
        """
        If `hash` is False on an attribute, ignore that attribute.
        """
        C = make_class("C", {"a": attr.ib(hash=False), "b": attr.ib()},
                       slots=slots, hash=True)

        assert hash(C(1, 2)) == hash(C(2, 2))

    @given(booleans())
    def test_hash_attribute_mirrors_cmp(self, cmp):
        """
        If `hash` is None, the hash generation mirrors `cmp`.
        """
        C = make_class("C", {"a": attr.ib(cmp=cmp)}, cmp=True, frozen=True)

        if cmp:
            assert C(1) != C(2)
            assert hash(C(1)) != hash(C(2))
            assert hash(C(1)) == hash(C(1))
        else:
            assert C(1) == C(2)
            assert hash(C(1)) == hash(C(2))

    @given(booleans())
    def test_hash_mirrors_cmp(self, cmp):
        """
        If `hash` is None, the hash generation mirrors `cmp`.
        """
        C = make_class("C", {"a": attr.ib()}, cmp=cmp, frozen=True)

        i = C(1)

        assert i == i
        assert hash(i) == hash(i)

        if cmp:
            assert C(1) == C(1)
            assert hash(C(1)) == hash(C(1))
        else:
            assert C(1) != C(1)
            assert hash(C(1)) != hash(C(1))

    @pytest.mark.parametrize("cls", [HashC, HashCSlots])
    def test_hash_works(self, cls):
        """
        __hash__ returns different hashes for different values.
        """
        assert hash(cls(1, 2)) != hash(cls(1, 1))

    def test_hash_default(self):
        """
        Classes are not hashable by default.
        """
        C = make_class("C", {})

        with pytest.raises(TypeError) as e:
            hash(C())

        assert e.value.args[0] in (
            "'C' objects are unhashable",  # PyPy
            "unhashable type: 'C'",  # CPython
        )


class TestAddInit(object):
    """
    Tests for `_add_init`.
    """
    @given(booleans(), booleans())
    def test_init(self, slots, frozen):
        """
        If `init` is False, ignore that attribute.
        """
        C = make_class("C", {"a": attr.ib(init=False), "b": attr.ib()},
                       slots=slots, frozen=frozen)
        with pytest.raises(TypeError) as e:
            C(a=1, b=2)

        assert (
            "__init__() got an unexpected keyword argument 'a'" ==
            e.value.args[0]
        )

    @given(booleans(), booleans())
    def test_no_init_default(self, slots, frozen):
        """
        If `init` is False but a Factory is specified, don't allow passing that
        argument but initialize it anyway.
        """
        C = make_class("C", {
            "_a": attr.ib(init=False, default=42),
            "_b": attr.ib(init=False, default=Factory(list)),
            "c": attr.ib()
        }, slots=slots, frozen=frozen)
        with pytest.raises(TypeError):
            C(a=1, c=2)
        with pytest.raises(TypeError):
            C(b=1, c=2)

        i = C(23)
        assert (42, [], 23) == (i._a, i._b, i.c)

    @given(booleans(), booleans())
    def test_no_init_order(self, slots, frozen):
        """
        If an attribute is `init=False`, it's legal to come after a mandatory
        attribute.
        """
        make_class("C", {
            "a": attr.ib(default=Factory(list)),
            "b": attr.ib(init=False),
        }, slots=slots, frozen=frozen)

    def test_sets_attributes(self):
        """
        The attributes are initialized using the passed keywords.
        """
        obj = InitC(a=1, b=2)
        assert 1 == obj.a
        assert 2 == obj.b

    def test_default(self):
        """
        If a default value is present, it's used as fallback.
        """
        class C(object):
            __attrs_attrs__ = [
                simple_attr(name="a", default=2),
                simple_attr(name="b", default="hallo"),
                simple_attr(name="c", default=None),
            ]

        C = _add_init(C, False)
        i = C()
        assert 2 == i.a
        assert "hallo" == i.b
        assert None is i.c

    def test_factory(self):
        """
        If a default factory is present, it's used as fallback.
        """
        class D(object):
            pass

        class C(object):
            __attrs_attrs__ = [
                simple_attr(name="a", default=Factory(list)),
                simple_attr(name="b", default=Factory(D)),
            ]
        C = _add_init(C, False)
        i = C()

        assert [] == i.a
        assert isinstance(i.b, D)

    def test_validator(self):
        """
        If a validator is passed, call it with the preliminary instance, the
        Attribute, and the argument.
        """
        class VException(Exception):
            pass

        def raiser(*args):
            raise VException(*args)

        C = make_class("C", {"a": attr.ib("a", validator=raiser)})
        with pytest.raises(VException) as e:
            C(42)

        assert (fields(C).a, 42,) == e.value.args[1:]
        assert isinstance(e.value.args[0], C)

    def test_validator_slots(self):
        """
        If a validator is passed, call it with the preliminary instance, the
        Attribute, and the argument.
        """
        class VException(Exception):
            pass

        def raiser(*args):
            raise VException(*args)

        C = make_class("C", {"a": attr.ib("a", validator=raiser)}, slots=True)
        with pytest.raises(VException) as e:
            C(42)

        assert (fields(C)[0], 42,) == e.value.args[1:]
        assert isinstance(e.value.args[0], C)

    @given(booleans())
    def test_validator_others(self, slots):
        """
        Does not interfere when setting non-attrs attributes.
        """
        C = make_class("C", {
            "a": attr.ib("a", validator=instance_of(int))
        }, slots=slots)
        i = C(1)

        assert 1 == i.a

        if not slots:
            i.b = "foo"
            assert "foo" == i.b
        else:
            with pytest.raises(AttributeError):
                i.b = "foo"

    def test_underscores(self):
        """
        The argument names in `__init__` are without leading and trailing
        underscores.
        """
        class C(object):
            __attrs_attrs__ = [simple_attr("_private")]

        C = _add_init(C, False)
        i = C(private=42)
        assert 42 == i._private


class TestNothing(object):
    """
    Tests for `_Nothing`.
    """
    def test_copy(self):
        """
        __copy__ returns the same object.
        """
        n = _Nothing()
        assert n is copy.copy(n)

    def test_deepcopy(self):
        """
        __deepcopy__ returns the same object.
        """
        n = _Nothing()
        assert n is copy.deepcopy(n)

    def test_eq(self):
        """
        All instances are equal.
        """
        assert _Nothing() == _Nothing() == NOTHING
        assert not (_Nothing() != _Nothing())
        assert 1 != _Nothing()
