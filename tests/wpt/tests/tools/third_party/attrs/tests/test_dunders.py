# SPDX-License-Identifier: MIT

"""
Tests for dunder methods from `attrib._make`.
"""


import copy
import inspect
import pickle

import pytest

from hypothesis import given
from hypothesis.strategies import booleans

import attr

from attr._make import (
    NOTHING,
    Factory,
    _add_repr,
    _is_slot_cls,
    _make_init,
    fields,
    make_class,
)
from attr.validators import instance_of

from .utils import simple_attr, simple_class


EqC = simple_class(eq=True)
EqCSlots = simple_class(eq=True, slots=True)
OrderC = simple_class(order=True)
OrderCSlots = simple_class(order=True, slots=True)
ReprC = simple_class(repr=True)
ReprCSlots = simple_class(repr=True, slots=True)


@attr.s(eq=True)
class EqCallableC:
    a = attr.ib(eq=str.lower, order=False)
    b = attr.ib(eq=True)


@attr.s(eq=True, slots=True)
class EqCallableCSlots:
    a = attr.ib(eq=str.lower, order=False)
    b = attr.ib(eq=True)


@attr.s(order=True)
class OrderCallableC:
    a = attr.ib(eq=True, order=str.lower)
    b = attr.ib(order=True)


@attr.s(order=True, slots=True)
class OrderCallableCSlots:
    a = attr.ib(eq=True, order=str.lower)
    b = attr.ib(order=True)


# HashC is hashable by explicit definition while HashCSlots is hashable
# implicitly.  The "Cached" versions are the same, except with hash code
# caching enabled
HashC = simple_class(hash=True)
HashCSlots = simple_class(hash=None, eq=True, frozen=True, slots=True)
HashCCached = simple_class(hash=True, cache_hash=True)
HashCSlotsCached = simple_class(
    hash=None, eq=True, frozen=True, slots=True, cache_hash=True
)
# the cached hash code is stored slightly differently in this case
# so it needs to be tested separately
HashCFrozenNotSlotsCached = simple_class(
    frozen=True, slots=False, hash=True, cache_hash=True
)


def _add_init(cls, frozen):
    """
    Add a __init__ method to *cls*.  If *frozen* is True, make it immutable.

    This function used to be part of _make.  It wasn't used anymore however
    the tests for it are still useful to test the behavior of _make_init.
    """
    has_pre_init = bool(getattr(cls, "__attrs_pre_init__", False))

    cls.__init__ = _make_init(
        cls,
        cls.__attrs_attrs__,
        has_pre_init,
        len(inspect.signature(cls.__attrs_pre_init__).parameters) > 1
        if has_pre_init
        else False,
        getattr(cls, "__attrs_post_init__", False),
        frozen,
        _is_slot_cls(cls),
        cache_hash=False,
        base_attr_map={},
        is_exc=False,
        cls_on_setattr=None,
        attrs_init=False,
    )
    return cls


class InitC:
    __attrs_attrs__ = [simple_attr("a"), simple_attr("b")]


InitC = _add_init(InitC, False)


class TestEqOrder:
    """
    Tests for eq and order related methods.
    """

    @given(booleans())
    def test_eq_ignore_attrib(self, slots):
        """
        If `eq` is False for an attribute, ignore that attribute.
        """
        C = make_class(
            "C", {"a": attr.ib(eq=False), "b": attr.ib()}, slots=slots
        )

        assert C(1, 2) == C(2, 2)

    @pytest.mark.parametrize("cls", [EqC, EqCSlots])
    def test_equal(self, cls):
        """
        Equal objects are detected as equal.
        """
        assert cls(1, 2) == cls(1, 2)
        assert not (cls(1, 2) != cls(1, 2))

    @pytest.mark.parametrize("cls", [EqCallableC, EqCallableCSlots])
    def test_equal_callable(self, cls):
        """
        Equal objects are detected as equal.
        """
        assert cls("Test", 1) == cls("test", 1)
        assert cls("Test", 1) != cls("test", 2)
        assert not (cls("Test", 1) != cls("test", 1))
        assert not (cls("Test", 1) == cls("test", 2))

    @pytest.mark.parametrize("cls", [EqC, EqCSlots])
    def test_unequal_same_class(self, cls):
        """
        Unequal objects of correct type are detected as unequal.
        """
        assert cls(1, 2) != cls(2, 1)
        assert not (cls(1, 2) == cls(2, 1))

    @pytest.mark.parametrize("cls", [EqCallableC, EqCallableCSlots])
    def test_unequal_same_class_callable(self, cls):
        """
        Unequal objects of correct type are detected as unequal.
        """
        assert cls("Test", 1) != cls("foo", 2)
        assert not (cls("Test", 1) == cls("foo", 2))

    @pytest.mark.parametrize(
        "cls", [EqC, EqCSlots, EqCallableC, EqCallableCSlots]
    )
    def test_unequal_different_class(self, cls):
        """
        Unequal objects of different type are detected even if their attributes
        match.
        """

        class NotEqC:
            a = 1
            b = 2

        assert cls(1, 2) != NotEqC()
        assert not (cls(1, 2) == NotEqC())

    @pytest.mark.parametrize("cls", [OrderC, OrderCSlots])
    def test_lt(self, cls):
        """
        __lt__ compares objects as tuples of attribute values.
        """
        for a, b in [
            ((1, 2), (2, 1)),
            ((1, 2), (1, 3)),
            (("a", "b"), ("b", "a")),
        ]:
            assert cls(*a) < cls(*b)

    @pytest.mark.parametrize("cls", [OrderCallableC, OrderCallableCSlots])
    def test_lt_callable(self, cls):
        """
        __lt__ compares objects as tuples of attribute values.
        """
        # Note: "A" < "a"
        for a, b in [
            (("test1", 1), ("Test1", 2)),
            (("test0", 1), ("Test1", 1)),
        ]:
            assert cls(*a) < cls(*b)

    @pytest.mark.parametrize(
        "cls", [OrderC, OrderCSlots, OrderCallableC, OrderCallableCSlots]
    )
    def test_lt_unordable(self, cls):
        """
        __lt__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__lt__(42))

    @pytest.mark.parametrize("cls", [OrderC, OrderCSlots])
    def test_le(self, cls):
        """
        __le__ compares objects as tuples of attribute values.
        """
        for a, b in [
            ((1, 2), (2, 1)),
            ((1, 2), (1, 3)),
            ((1, 1), (1, 1)),
            (("a", "b"), ("b", "a")),
            (("a", "b"), ("a", "b")),
        ]:
            assert cls(*a) <= cls(*b)

    @pytest.mark.parametrize("cls", [OrderCallableC, OrderCallableCSlots])
    def test_le_callable(self, cls):
        """
        __le__ compares objects as tuples of attribute values.
        """
        # Note: "A" < "a"
        for a, b in [
            (("test1", 1), ("Test1", 1)),
            (("test1", 1), ("Test1", 2)),
            (("test0", 1), ("Test1", 1)),
            (("test0", 2), ("Test1", 1)),
        ]:
            assert cls(*a) <= cls(*b)

    @pytest.mark.parametrize(
        "cls", [OrderC, OrderCSlots, OrderCallableC, OrderCallableCSlots]
    )
    def test_le_unordable(self, cls):
        """
        __le__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__le__(42))

    @pytest.mark.parametrize("cls", [OrderC, OrderCSlots])
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

    @pytest.mark.parametrize("cls", [OrderCallableC, OrderCallableCSlots])
    def test_gt_callable(self, cls):
        """
        __gt__ compares objects as tuples of attribute values.
        """
        # Note: "A" < "a"
        for a, b in [
            (("Test1", 2), ("test1", 1)),
            (("Test1", 1), ("test0", 1)),
        ]:
            assert cls(*a) > cls(*b)

    @pytest.mark.parametrize(
        "cls", [OrderC, OrderCSlots, OrderCallableC, OrderCallableCSlots]
    )
    def test_gt_unordable(self, cls):
        """
        __gt__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__gt__(42))

    @pytest.mark.parametrize("cls", [OrderC, OrderCSlots])
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

    @pytest.mark.parametrize("cls", [OrderCallableC, OrderCallableCSlots])
    def test_ge_callable(self, cls):
        """
        __ge__ compares objects as tuples of attribute values.
        """
        # Note: "A" < "a"
        for a, b in [
            (("Test1", 1), ("test1", 1)),
            (("Test1", 2), ("test1", 1)),
            (("Test1", 1), ("test0", 1)),
            (("Test1", 1), ("test0", 2)),
        ]:
            assert cls(*a) >= cls(*b)

    @pytest.mark.parametrize(
        "cls", [OrderC, OrderCSlots, OrderCallableC, OrderCallableCSlots]
    )
    def test_ge_unordable(self, cls):
        """
        __ge__ returns NotImplemented if classes differ.
        """
        assert NotImplemented == (cls(1, 2).__ge__(42))


class TestAddRepr:
    """
    Tests for `_add_repr`.
    """

    def test_repr(self, slots):
        """
        If `repr` is False, ignore that attribute.
        """
        C = make_class(
            "C", {"a": attr.ib(repr=False), "b": attr.ib()}, slots=slots
        )

        assert "C(b=2)" == repr(C(1, 2))

    @pytest.mark.parametrize("cls", [ReprC, ReprCSlots])
    def test_repr_works(self, cls):
        """
        repr returns a sensible value.
        """
        assert "C(a=1, b=2)" == repr(cls(1, 2))

    def test_custom_repr_works(self):
        """
        repr returns a sensible value for attributes with a custom repr
        callable.
        """

        def custom_repr(value):
            return "foo:" + str(value)

        @attr.s
        class C:
            a = attr.ib(repr=custom_repr)

        assert "C(a=foo:1)" == repr(C(1))

    def test_infinite_recursion(self):
        """
        In the presence of a cyclic graph, repr will emit an ellipsis and not
        raise an exception.
        """

        @attr.s
        class Cycle:
            value = attr.ib(default=7)
            cycle = attr.ib(default=None)

        cycle = Cycle()
        cycle.cycle = cycle
        assert "Cycle(value=7, cycle=...)" == repr(cycle)

    def test_infinite_recursion_long_cycle(self):
        """
        A cyclic graph can pass through other non-attrs objects, and repr will
        still emit an ellipsis and not raise an exception.
        """

        @attr.s
        class LongCycle:
            value = attr.ib(default=14)
            cycle = attr.ib(default=None)

        cycle = LongCycle()
        # Ensure that the reference cycle passes through a non-attrs object.
        # This demonstrates the need for a thread-local "global" ID tracker.
        cycle.cycle = {"cycle": [cycle]}
        assert "LongCycle(value=14, cycle={'cycle': [...]})" == repr(cycle)

    def test_underscores(self):
        """
        repr does not strip underscores.
        """

        class C:
            __attrs_attrs__ = [simple_attr("_x")]

        C = _add_repr(C)
        i = C()
        i._x = 42

        assert "C(_x=42)" == repr(i)

    def test_repr_uninitialized_member(self):
        """
        repr signals unset attributes
        """
        C = make_class("C", {"a": attr.ib(init=False)})

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


# these are for use in TestAddHash.test_cache_hash_serialization
# they need to be out here so they can be un-pickled
@attr.attrs(hash=True, cache_hash=False)
class HashCacheSerializationTestUncached:
    foo_value = attr.ib()


@attr.attrs(hash=True, cache_hash=True)
class HashCacheSerializationTestCached:
    foo_value = attr.ib()


@attr.attrs(slots=True, hash=True, cache_hash=True)
class HashCacheSerializationTestCachedSlots:
    foo_value = attr.ib()


class IncrementingHasher:
    def __init__(self):
        self.hash_value = 100

    def __hash__(self):
        rv = self.hash_value
        self.hash_value += 1
        return rv


class TestAddHash:
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

    def test_enforce_no_cache_hash_without_hash(self):
        """
        Ensure exception is thrown if caching the hash code is requested
        but attrs is not requested to generate `__hash__`.
        """
        exc_args = (
            "Invalid value for cache_hash.  To use hash caching,"
            " hashing must be either explicitly or implicitly "
            "enabled.",
        )
        with pytest.raises(TypeError) as e:
            make_class("C", {}, hash=False, cache_hash=True)
        assert exc_args == e.value.args

        # unhashable case
        with pytest.raises(TypeError) as e:
            make_class(
                "C", {}, hash=None, eq=True, frozen=False, cache_hash=True
            )
        assert exc_args == e.value.args

    def test_enforce_no_cached_hash_without_init(self):
        """
        Ensure exception is thrown if caching the hash code is requested
        but attrs is not requested to generate `__init__`.
        """
        exc_args = (
            "Invalid value for cache_hash.  To use hash caching,"
            " init must be True.",
        )
        with pytest.raises(TypeError) as e:
            make_class("C", {}, init=False, hash=True, cache_hash=True)
        assert exc_args == e.value.args

    @given(booleans(), booleans())
    def test_hash_attribute(self, slots, cache_hash):
        """
        If `hash` is False on an attribute, ignore that attribute.
        """
        C = make_class(
            "C",
            {"a": attr.ib(hash=False), "b": attr.ib()},
            slots=slots,
            hash=True,
            cache_hash=cache_hash,
        )

        assert hash(C(1, 2)) == hash(C(2, 2))

    @given(booleans())
    def test_hash_attribute_mirrors_eq(self, eq):
        """
        If `hash` is None, the hash generation mirrors `eq`.
        """
        C = make_class("C", {"a": attr.ib(eq=eq)}, eq=True, frozen=True)

        if eq:
            assert C(1) != C(2)
            assert hash(C(1)) != hash(C(2))
            assert hash(C(1)) == hash(C(1))
        else:
            assert C(1) == C(2)
            assert hash(C(1)) == hash(C(2))

    @given(booleans())
    def test_hash_mirrors_eq(self, eq):
        """
        If `hash` is None, the hash generation mirrors `eq`.
        """
        C = make_class("C", {"a": attr.ib()}, eq=eq, frozen=True)

        i = C(1)

        assert i == i
        assert hash(i) == hash(i)

        if eq:
            assert C(1) == C(1)
            assert hash(C(1)) == hash(C(1))
        else:
            assert C(1) != C(1)
            assert hash(C(1)) != hash(C(1))

    @pytest.mark.parametrize(
        "cls",
        [
            HashC,
            HashCSlots,
            HashCCached,
            HashCSlotsCached,
            HashCFrozenNotSlotsCached,
        ],
    )
    def test_hash_works(self, cls):
        """
        __hash__ returns different hashes for different values.
        """
        a = cls(1, 2)
        b = cls(1, 1)
        assert hash(a) != hash(b)
        # perform the test again to test the pre-cached path through
        # __hash__ for the cached-hash versions
        assert hash(a) != hash(b)

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

    def test_cache_hashing(self):
        """
        Ensure that hash computation if cached if and only if requested
        """

        class HashCounter:
            """
            A class for testing which counts how many times its hash
            has been requested
            """

            def __init__(self):
                self.times_hash_called = 0

            def __hash__(self):
                self.times_hash_called += 1
                return 12345

        Uncached = make_class(
            "Uncached",
            {"hash_counter": attr.ib(factory=HashCounter)},
            hash=True,
            cache_hash=False,
        )
        Cached = make_class(
            "Cached",
            {"hash_counter": attr.ib(factory=HashCounter)},
            hash=True,
            cache_hash=True,
        )

        uncached_instance = Uncached()
        cached_instance = Cached()

        hash(uncached_instance)
        hash(uncached_instance)
        hash(cached_instance)
        hash(cached_instance)

        assert 2 == uncached_instance.hash_counter.times_hash_called
        assert 1 == cached_instance.hash_counter.times_hash_called

    @pytest.mark.parametrize("cache_hash", [True, False])
    def test_copy_hash_cleared(self, cache_hash, frozen, slots):
        """
        Test that the default hash is recalculated after a copy operation.
        """

        kwargs = {"frozen": frozen, "slots": slots, "cache_hash": cache_hash}

        # Give it an explicit hash if we don't have an implicit one
        if not frozen:
            kwargs["hash"] = True

        @attr.s(**kwargs)
        class C:
            x = attr.ib()

        a = C(IncrementingHasher())
        # Ensure that any hash cache would be calculated before copy
        orig_hash = hash(a)
        b = copy.deepcopy(a)

        if kwargs["cache_hash"]:
            # For cache_hash classes, this call is cached
            assert orig_hash == hash(a)

        assert orig_hash != hash(b)

    @pytest.mark.parametrize(
        ("klass", "cached"),
        [
            (HashCacheSerializationTestUncached, False),
            (HashCacheSerializationTestCached, True),
            (HashCacheSerializationTestCachedSlots, True),
        ],
    )
    def test_cache_hash_serialization_hash_cleared(self, klass, cached):
        """
        Tests that the hash cache is cleared on deserialization to fix
        https://github.com/python-attrs/attrs/issues/482 .

        This test is intended to guard against a stale hash code surviving
        across serialization (which may cause problems when the hash value
        is different in different interpreters).
        """

        obj = klass(IncrementingHasher())
        original_hash = hash(obj)
        obj_rt = self._roundtrip_pickle(obj)

        if cached:
            assert original_hash == hash(obj)

        assert original_hash != hash(obj_rt)

    def test_copy_two_arg_reduce(self, frozen):
        """
        If __getstate__ returns None, the tuple returned by object.__reduce__
        won't contain the state dictionary; this test ensures that the custom
        __reduce__ generated when cache_hash=True works in that case.
        """

        @attr.s(frozen=frozen, cache_hash=True, hash=True)
        class C:
            x = attr.ib()

            def __getstate__(self):
                return None

        # By the nature of this test it doesn't really create an object that's
        # in a valid state - it basically does the equivalent of
        # `object.__new__(C)`, so it doesn't make much sense to assert anything
        # about the result of the copy. This test will just check that it
        # doesn't raise an *error*.
        copy.deepcopy(C(1))

    def _roundtrip_pickle(self, obj):
        pickle_str = pickle.dumps(obj)
        return pickle.loads(pickle_str)


class TestAddInit:
    """
    Tests for `_add_init`.
    """

    @given(booleans(), booleans())
    def test_init(self, slots, frozen):
        """
        If `init` is False, ignore that attribute.
        """
        C = make_class(
            "C",
            {"a": attr.ib(init=False), "b": attr.ib()},
            slots=slots,
            frozen=frozen,
        )
        with pytest.raises(TypeError) as e:
            C(a=1, b=2)

        assert e.value.args[0].endswith(
            "__init__() got an unexpected keyword argument 'a'"
        )

    @given(booleans(), booleans())
    def test_no_init_default(self, slots, frozen):
        """
        If `init` is False but a Factory is specified, don't allow passing that
        argument but initialize it anyway.
        """
        C = make_class(
            "C",
            {
                "_a": attr.ib(init=False, default=42),
                "_b": attr.ib(init=False, default=Factory(list)),
                "c": attr.ib(),
            },
            slots=slots,
            frozen=frozen,
        )
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
        make_class(
            "C",
            {"a": attr.ib(default=Factory(list)), "b": attr.ib(init=False)},
            slots=slots,
            frozen=frozen,
        )

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

        class C:
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

        class D:
            pass

        class C:
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

        assert (fields(C).a, 42) == e.value.args[1:]
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

        assert (fields(C)[0], 42) == e.value.args[1:]
        assert isinstance(e.value.args[0], C)

    @given(booleans())
    def test_validator_others(self, slots):
        """
        Does not interfere when setting non-attrs attributes.
        """
        C = make_class(
            "C", {"a": attr.ib("a", validator=instance_of(int))}, slots=slots
        )
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

        class C:
            __attrs_attrs__ = [simple_attr("_private")]

        C = _add_init(C, False)
        i = C(private=42)
        assert 42 == i._private


class TestNothing:
    """
    Tests for `NOTHING`.
    """

    def test_copy(self):
        """
        __copy__ returns the same object.
        """
        n = NOTHING
        assert n is copy.copy(n)

    def test_deepcopy(self):
        """
        __deepcopy__ returns the same object.
        """
        n = NOTHING
        assert n is copy.deepcopy(n)

    def test_eq(self):
        """
        All instances are equal.
        """
        assert NOTHING == NOTHING == NOTHING
        assert not (NOTHING != NOTHING)
        assert 1 != NOTHING

    def test_false(self):
        """
        NOTHING evaluates as falsey.
        """
        assert not NOTHING
        assert False is bool(NOTHING)


@attr.s(hash=True, order=True)
class C:
    pass


# Store this class so that we recreate it.
OriginalC = C


@attr.s(hash=True, order=True)
class C:
    pass


CopyC = C


@attr.s(hash=True, order=True)
class C:
    """A different class, to generate different methods."""

    a = attr.ib()


class TestFilenames:
    def test_filenames(self):
        """
        The created dunder methods have a "consistent" filename.
        """
        assert (
            OriginalC.__init__.__code__.co_filename
            == "<attrs generated init tests.test_dunders.C>"
        )
        assert (
            OriginalC.__eq__.__code__.co_filename
            == "<attrs generated eq tests.test_dunders.C>"
        )
        assert (
            OriginalC.__hash__.__code__.co_filename
            == "<attrs generated hash tests.test_dunders.C>"
        )
        assert (
            CopyC.__init__.__code__.co_filename
            == "<attrs generated init tests.test_dunders.C>"
        )
        assert (
            CopyC.__eq__.__code__.co_filename
            == "<attrs generated eq tests.test_dunders.C>"
        )
        assert (
            CopyC.__hash__.__code__.co_filename
            == "<attrs generated hash tests.test_dunders.C>"
        )
        assert (
            C.__init__.__code__.co_filename
            == "<attrs generated init tests.test_dunders.C-1>"
        )
        assert (
            C.__eq__.__code__.co_filename
            == "<attrs generated eq tests.test_dunders.C-1>"
        )
        assert (
            C.__hash__.__code__.co_filename
            == "<attrs generated hash tests.test_dunders.C-1>"
        )
