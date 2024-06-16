# SPDX-License-Identifier: MIT

"""
Unit tests for slots-related functionality.
"""
import functools
import pickle
import weakref

from unittest import mock

import pytest

import attr
import attrs

from attr._compat import PY_3_8_PLUS, PYPY


# Pympler doesn't work on PyPy.
try:
    from pympler.asizeof import asizeof

    has_pympler = True
except BaseException:  # Won't be an import error.  # noqa: BLE001
    has_pympler = False


@attr.s
class C1:
    x = attr.ib(validator=attr.validators.instance_of(int))
    y = attr.ib()

    def method(self):
        return self.x

    @classmethod
    def classmethod(cls):
        return "clsmethod"

    @staticmethod
    def staticmethod():
        return "staticmethod"

    def my_class(self):
        return __class__

    def my_super(self):
        """Just to test out the no-arg super."""
        return super().__repr__()


@attr.s(slots=True, hash=True)
class C1Slots:
    x = attr.ib(validator=attr.validators.instance_of(int))
    y = attr.ib()

    def method(self):
        return self.x

    @classmethod
    def classmethod(cls):
        return "clsmethod"

    @staticmethod
    def staticmethod():
        return "staticmethod"

    def my_class(self):
        return __class__

    def my_super(self):
        """Just to test out the no-arg super."""
        return super().__repr__()


def test_slots_being_used():
    """
    The class is really using __slots__.
    """
    non_slot_instance = C1(x=1, y="test")
    slot_instance = C1Slots(x=1, y="test")

    assert "__dict__" not in dir(slot_instance)
    assert "__slots__" in dir(slot_instance)

    assert "__dict__" in dir(non_slot_instance)
    assert "__slots__" not in dir(non_slot_instance)

    assert {"__weakref__", "x", "y"} == set(slot_instance.__slots__)

    if has_pympler:
        assert asizeof(slot_instance) < asizeof(non_slot_instance)

    non_slot_instance.t = "test"
    with pytest.raises(AttributeError):
        slot_instance.t = "test"

    assert 1 == non_slot_instance.method()
    assert 1 == slot_instance.method()

    assert attr.fields(C1Slots) == attr.fields(C1)
    assert attr.asdict(slot_instance) == attr.asdict(non_slot_instance)


def test_basic_attr_funcs():
    """
    Comparison, `__eq__`, `__hash__`, `__repr__`, `attrs.asdict` work.
    """
    a = C1Slots(x=1, y=2)
    b = C1Slots(x=1, y=3)
    a_ = C1Slots(x=1, y=2)

    # Comparison.
    assert b > a

    assert a_ == a

    # Hashing.
    hash(b)  # Just to assert it doesn't raise.

    # Repr.
    assert "C1Slots(x=1, y=2)" == repr(a)

    assert {"x": 1, "y": 2} == attr.asdict(a)


def test_inheritance_from_nonslots():
    """
    Inheritance from a non-slotted class works.

    Note that a slotted class inheriting from an ordinary class loses most of
    the benefits of slotted classes, but it should still work.
    """

    @attr.s(slots=True, hash=True)
    class C2Slots(C1):
        z = attr.ib()

    c2 = C2Slots(x=1, y=2, z="test")

    assert 1 == c2.x
    assert 2 == c2.y
    assert "test" == c2.z

    c2.t = "test"  # This will work, using the base class.

    assert "test" == c2.t

    assert 1 == c2.method()
    assert "clsmethod" == c2.classmethod()
    assert "staticmethod" == c2.staticmethod()

    assert {"z"} == set(C2Slots.__slots__)

    c3 = C2Slots(x=1, y=3, z="test")

    assert c3 > c2

    c2_ = C2Slots(x=1, y=2, z="test")

    assert c2 == c2_

    assert "C2Slots(x=1, y=2, z='test')" == repr(c2)

    hash(c2)  # Just to assert it doesn't raise.

    assert {"x": 1, "y": 2, "z": "test"} == attr.asdict(c2)


def test_nonslots_these():
    """
    Enhancing a dict class using 'these' works.

    This will actually *replace* the class with another one, using slots.
    """

    class SimpleOrdinaryClass:
        def __init__(self, x, y, z):
            self.x = x
            self.y = y
            self.z = z

        def method(self):
            return self.x

        @classmethod
        def classmethod(cls):
            return "clsmethod"

        @staticmethod
        def staticmethod():
            return "staticmethod"

    C2Slots = attr.s(
        these={"x": attr.ib(), "y": attr.ib(), "z": attr.ib()},
        init=False,
        slots=True,
        hash=True,
    )(SimpleOrdinaryClass)

    c2 = C2Slots(x=1, y=2, z="test")
    assert 1 == c2.x
    assert 2 == c2.y
    assert "test" == c2.z
    with pytest.raises(AttributeError):
        c2.t = "test"  # We have slots now.

    assert 1 == c2.method()
    assert "clsmethod" == c2.classmethod()
    assert "staticmethod" == c2.staticmethod()

    assert {"__weakref__", "x", "y", "z"} == set(C2Slots.__slots__)

    c3 = C2Slots(x=1, y=3, z="test")
    assert c3 > c2
    c2_ = C2Slots(x=1, y=2, z="test")
    assert c2 == c2_

    assert "SimpleOrdinaryClass(x=1, y=2, z='test')" == repr(c2)

    hash(c2)  # Just to assert it doesn't raise.

    assert {"x": 1, "y": 2, "z": "test"} == attr.asdict(c2)


def test_inheritance_from_slots():
    """
    Inheriting from an attrs slotted class works.
    """

    @attr.s(slots=True, hash=True)
    class C2Slots(C1Slots):
        z = attr.ib()

    @attr.s(slots=True, hash=True)
    class C2(C1):
        z = attr.ib()

    c2 = C2Slots(x=1, y=2, z="test")
    assert 1 == c2.x
    assert 2 == c2.y
    assert "test" == c2.z

    assert {"z"} == set(C2Slots.__slots__)

    assert 1 == c2.method()
    assert "clsmethod" == c2.classmethod()
    assert "staticmethod" == c2.staticmethod()

    with pytest.raises(AttributeError):
        c2.t = "test"

    non_slot_instance = C2(x=1, y=2, z="test")
    if has_pympler:
        assert asizeof(c2) < asizeof(non_slot_instance)

    c3 = C2Slots(x=1, y=3, z="test")
    assert c3 > c2
    c2_ = C2Slots(x=1, y=2, z="test")
    assert c2 == c2_

    assert "C2Slots(x=1, y=2, z='test')" == repr(c2)

    hash(c2)  # Just to assert it doesn't raise.

    assert {"x": 1, "y": 2, "z": "test"} == attr.asdict(c2)


def test_inheritance_from_slots_with_attribute_override():
    """
    Inheriting from a slotted class doesn't re-create existing slots
    """

    class HasXSlot:
        __slots__ = ("x",)

    @attr.s(slots=True, hash=True)
    class C2Slots(C1Slots):
        # y re-defined here but it shouldn't get a slot
        y = attr.ib()
        z = attr.ib()

    @attr.s(slots=True, hash=True)
    class NonAttrsChild(HasXSlot):
        # Parent class has slot for "x" already, so we skip it
        x = attr.ib()
        y = attr.ib()
        z = attr.ib()

    c2 = C2Slots(1, 2, "test")
    assert 1 == c2.x
    assert 2 == c2.y
    assert "test" == c2.z

    assert {"z"} == set(C2Slots.__slots__)

    na = NonAttrsChild(1, 2, "test")
    assert 1 == na.x
    assert 2 == na.y
    assert "test" == na.z

    assert {"__weakref__", "y", "z"} == set(NonAttrsChild.__slots__)


def test_inherited_slot_reuses_slot_descriptor():
    """
    We reuse slot descriptor for an attr.ib defined in a slotted attr.s
    """

    class HasXSlot:
        __slots__ = ("x",)

    class OverridesX(HasXSlot):
        @property
        def x(self):
            return None

    @attr.s(slots=True)
    class Child(OverridesX):
        x = attr.ib()

    assert Child.x is not OverridesX.x
    assert Child.x is HasXSlot.x

    c = Child(1)
    assert 1 == c.x
    assert set() == set(Child.__slots__)

    ox = OverridesX()
    assert ox.x is None


def test_bare_inheritance_from_slots():
    """
    Inheriting from a bare attrs slotted class works.
    """

    @attr.s(
        init=False, eq=False, order=False, hash=False, repr=False, slots=True
    )
    class C1BareSlots:
        x = attr.ib(validator=attr.validators.instance_of(int))
        y = attr.ib()

        def method(self):
            return self.x

        @classmethod
        def classmethod(cls):
            return "clsmethod"

        @staticmethod
        def staticmethod():
            return "staticmethod"

    @attr.s(init=False, eq=False, order=False, hash=False, repr=False)
    class C1Bare:
        x = attr.ib(validator=attr.validators.instance_of(int))
        y = attr.ib()

        def method(self):
            return self.x

        @classmethod
        def classmethod(cls):
            return "clsmethod"

        @staticmethod
        def staticmethod():
            return "staticmethod"

    @attr.s(slots=True, hash=True)
    class C2Slots(C1BareSlots):
        z = attr.ib()

    @attr.s(slots=True, hash=True)
    class C2(C1Bare):
        z = attr.ib()

    c2 = C2Slots(x=1, y=2, z="test")
    assert 1 == c2.x
    assert 2 == c2.y
    assert "test" == c2.z

    assert 1 == c2.method()
    assert "clsmethod" == c2.classmethod()
    assert "staticmethod" == c2.staticmethod()

    with pytest.raises(AttributeError):
        c2.t = "test"

    non_slot_instance = C2(x=1, y=2, z="test")
    if has_pympler:
        assert asizeof(c2) < asizeof(non_slot_instance)

    c3 = C2Slots(x=1, y=3, z="test")
    assert c3 > c2
    c2_ = C2Slots(x=1, y=2, z="test")
    assert c2 == c2_

    assert "C2Slots(x=1, y=2, z='test')" == repr(c2)

    hash(c2)  # Just to assert it doesn't raise.

    assert {"x": 1, "y": 2, "z": "test"} == attr.asdict(c2)


class TestClosureCellRewriting:
    def test_closure_cell_rewriting(self):
        """
        Slotted classes support proper closure cell rewriting.

        This affects features like `__class__` and the no-arg super().
        """
        non_slot_instance = C1(x=1, y="test")
        slot_instance = C1Slots(x=1, y="test")

        assert non_slot_instance.my_class() is C1
        assert slot_instance.my_class() is C1Slots

        # Just assert they return something, and not an exception.
        assert non_slot_instance.my_super()
        assert slot_instance.my_super()

    def test_inheritance(self):
        """
        Slotted classes support proper closure cell rewriting when inheriting.

        This affects features like `__class__` and the no-arg super().
        """

        @attr.s
        class C2(C1):
            def my_subclass(self):
                return __class__

        @attr.s
        class C2Slots(C1Slots):
            def my_subclass(self):
                return __class__

        non_slot_instance = C2(x=1, y="test")
        slot_instance = C2Slots(x=1, y="test")

        assert non_slot_instance.my_class() is C1
        assert slot_instance.my_class() is C1Slots

        # Just assert they return something, and not an exception.
        assert non_slot_instance.my_super()
        assert slot_instance.my_super()

        assert non_slot_instance.my_subclass() is C2
        assert slot_instance.my_subclass() is C2Slots

    def test_cls_static(self, slots):
        """
        Slotted classes support proper closure cell rewriting for class- and
        static methods.
        """
        # Python can reuse closure cells, so we create new classes just for
        # this test.

        @attr.s(slots=slots)
        class C:
            @classmethod
            def clsmethod(cls):
                return __class__

        assert C.clsmethod() is C

        @attr.s(slots=slots)
        class D:
            @staticmethod
            def statmethod():
                return __class__

        assert D.statmethod() is D


@pytest.mark.skipif(PYPY, reason="__slots__ only block weakref on CPython")
def test_not_weakrefable():
    """
    Instance is not weak-referenceable when `weakref_slot=False` in CPython.
    """

    @attr.s(slots=True, weakref_slot=False)
    class C:
        pass

    c = C()

    with pytest.raises(TypeError):
        weakref.ref(c)


@pytest.mark.skipif(
    not PYPY, reason="slots without weakref_slot should only work on PyPy"
)
def test_implicitly_weakrefable():
    """
    Instance is weak-referenceable even when `weakref_slot=False` in PyPy.
    """

    @attr.s(slots=True, weakref_slot=False)
    class C:
        pass

    c = C()
    w = weakref.ref(c)

    assert c is w()


def test_weakrefable():
    """
    Instance is weak-referenceable when `weakref_slot=True`.
    """

    @attr.s(slots=True, weakref_slot=True)
    class C:
        pass

    c = C()
    w = weakref.ref(c)

    assert c is w()


def test_weakref_does_not_add_a_field():
    """
    `weakref_slot=True` does not add a field to the class.
    """

    @attr.s(slots=True, weakref_slot=True)
    class C:
        field = attr.ib()

    assert [f.name for f in attr.fields(C)] == ["field"]


def tests_weakref_does_not_add_when_inheriting_with_weakref():
    """
    `weakref_slot=True` does not add a new __weakref__ slot when inheriting
    one.
    """

    @attr.s(slots=True, weakref_slot=True)
    class C:
        pass

    @attr.s(slots=True, weakref_slot=True)
    class D(C):
        pass

    d = D()
    w = weakref.ref(d)

    assert d is w()


def tests_weakref_does_not_add_with_weakref_attribute():
    """
    `weakref_slot=True` does not add a new __weakref__ slot when an attribute
    of that name exists.
    """

    @attr.s(slots=True, weakref_slot=True)
    class C:
        __weakref__ = attr.ib(
            init=False, hash=False, repr=False, eq=False, order=False
        )

    c = C()
    w = weakref.ref(c)

    assert c is w()


def test_slots_empty_cell():
    """
    Tests that no `ValueError: Cell is empty` exception is raised when
    closure cells are present with no contents in a `slots=True` class.
    (issue https://github.com/python-attrs/attrs/issues/589)

    If a method mentions `__class__` or uses the no-arg `super()`, the compiler
    will bake a reference to the class in the method itself as
    `method.__closure__`. Since `attrs` replaces the class with a clone,
    `_ClassBuilder._create_slots_class(self)` will rewrite these references so
    it keeps working. This method was not properly covering the edge case where
    the closure cell was empty, we fixed it and this is the non-regression
    test.
    """

    @attr.s(slots=True)
    class C:
        field = attr.ib()

        def f(self, a):
            super(C, self).__init__()  # noqa: UP008

    C(field=1)


@attr.s(getstate_setstate=True)
class C2:
    x = attr.ib()


@attr.s(slots=True, getstate_setstate=True)
class C2Slots:
    x = attr.ib()


class TestPickle:
    @pytest.mark.parametrize("protocol", range(pickle.HIGHEST_PROTOCOL))
    def test_pickleable_by_default(self, protocol):
        """
        If nothing else is passed, slotted classes can be pickled and
        unpickled with all supported protocols.
        """
        i1 = C1Slots(1, 2)
        i2 = pickle.loads(pickle.dumps(i1, protocol))

        assert i1 == i2
        assert i1 is not i2

    def test_no_getstate_setstate_for_dict_classes(self):
        """
        As long as getstate_setstate is None, nothing is done to dict
        classes.
        """
        assert getattr(object, "__getstate__", None) is getattr(
            C1, "__getstate__", None
        )
        assert getattr(object, "__setstate__", None) is getattr(
            C1, "__setstate__", None
        )

    def test_no_getstate_setstate_if_option_false(self):
        """
        Don't add getstate/setstate if getstate_setstate is False.
        """

        @attr.s(slots=True, getstate_setstate=False)
        class C:
            x = attr.ib()

        assert getattr(object, "__getstate__", None) is getattr(
            C, "__getstate__", None
        )
        assert getattr(object, "__setstate__", None) is getattr(
            C, "__setstate__", None
        )

    @pytest.mark.parametrize("cls", [C2(1), C2Slots(1)])
    def test_getstate_set_state_force_true(self, cls):
        """
        If getstate_setstate is True, add them unconditionally.
        """
        assert None is not getattr(cls, "__getstate__", None)
        assert None is not getattr(cls, "__setstate__", None)


def test_slots_super_property_get():
    """
    Both `super()` and `super(self.__class__, self)` work.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @property
        def f(self):
            return self.x

    @attr.s(slots=True)
    class B(A):
        @property
        def f(self):
            return super().f ** 2

    @attr.s(slots=True)
    class C(A):
        @property
        def f(self):
            return super(C, self).f ** 2  # noqa: UP008

    assert B(11).f == 121
    assert B(17).f == 289
    assert C(11).f == 121
    assert C(17).f == 289


def test_slots_super_property_get_shortcut():
    """
    The `super()` shortcut is allowed.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @property
        def f(self):
            return self.x

    @attr.s(slots=True)
    class B(A):
        @property
        def f(self):
            return super().f ** 2

    assert B(11).f == 121
    assert B(17).f == 289


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_allows_call():
    """
    cached_property in slotted class allows call.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    assert A(11).f == 11


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_class_does_not_have__dict__():
    """
    slotted class with cached property has no __dict__ attribute.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    assert set(A.__slots__) == {"x", "f", "__weakref__"}
    assert "__dict__" not in dir(A)


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_works_on_frozen_isntances():
    """
    Infers type of cached property.
    """

    @attrs.frozen(slots=True)
    class A:
        x: int

        @functools.cached_property
        def f(self) -> int:
            return self.x

    assert A(x=1).f == 1


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_infers_type():
    """
    Infers type of cached property.
    """

    @attrs.frozen(slots=True)
    class A:
        x: int

        @functools.cached_property
        def f(self) -> int:
            return self.x

    assert A.__annotations__ == {"x": int, "f": int}


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_with_empty_getattr_raises_attribute_error_of_requested():
    """
    Ensures error information is not lost.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    a = A(1)
    with pytest.raises(
        AttributeError, match="'A' object has no attribute 'z'"
    ):
        a.z


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_with_getattr_calls_getattr_for_missing_attributes():
    """
    Ensure __getattr__ implementation is maintained for non cached_properties.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

        def __getattr__(self, item):
            return item

    a = A(1)
    assert a.f == 1
    assert a.z == "z"


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_getattr_in_superclass__is_called_for_missing_attributes_when_cached_property_present():
    """
    Ensure __getattr__ implementation is maintained in subclass.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        def __getattr__(self, item):
            return item

    @attr.s(slots=True)
    class B(A):
        @functools.cached_property
        def f(self):
            return self.x

    b = B(1)
    assert b.f == 1
    assert b.z == "z"


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_getattr_in_subclass_gets_superclass_cached_property():
    """
    Ensure super() in __getattr__ is not broken through cached_property re-write.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

        def __getattr__(self, item):
            return item

    @attr.s(slots=True)
    class B(A):
        @functools.cached_property
        def g(self):
            return self.x

        def __getattr__(self, item):
            return super().__getattr__(item)

    b = B(1)
    assert b.f == 1
    assert b.z == "z"


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_sub_class_with_independent_cached_properties_both_work():
    """
    Subclassing shouldn't break cached properties.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    @attr.s(slots=True)
    class B(A):
        @functools.cached_property
        def g(self):
            return self.x * 2

    assert B(1).f == 1
    assert B(1).g == 2


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_with_multiple_cached_property_subclasses_works():
    """
    Multiple sub-classes shouldn't break cached properties.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib(kw_only=True)

        @functools.cached_property
        def f(self):
            return self.x

    @attr.s(slots=False)
    class B:
        @functools.cached_property
        def g(self):
            return self.x * 2

        def __getattr__(self, item):
            if hasattr(super(), "__getattr__"):
                return super().__getattr__(item)
            return item

    @attr.s(slots=True)
    class AB(A, B):
        pass

    ab = AB(x=1)

    assert ab.f == 1
    assert ab.g == 2
    assert ab.h == "h"


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_sub_class_avoids_duplicated_slots():
    """
    Duplicating the slots is a waste of memory.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    @attr.s(slots=True)
    class B(A):
        @functools.cached_property
        def f(self):
            return self.x * 2

    assert B(1).f == 2
    assert B.__slots__ == ()


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_sub_class_with_actual_slot():
    """
    A sub-class can have an explicit attrs field that replaces a cached property.
    """

    @attr.s(slots=True)
    class A:  # slots : (x, f)
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    @attr.s(slots=True)
    class B(A):
        f: int = attr.ib()

    assert B(1, 2).f == 2
    assert B.__slots__ == ()


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_is_not_called_at_construction():
    """
    A cached property function should only be called at property access point.
    """
    call_count = 0

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            nonlocal call_count
            call_count += 1
            return self.x

    A(1)
    assert call_count == 0


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_repeat_call_only_once():
    """
    A cached property function should be called only once, on repeated attribute access.
    """
    call_count = 0

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            nonlocal call_count
            call_count += 1
            return self.x

    obj = A(1)
    obj.f
    obj.f
    assert call_count == 1


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_property_called_independent_across_instances():
    """
    A cached property value should be specific to the given instance.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f(self):
            return self.x

    obj_1 = A(1)
    obj_2 = A(2)

    assert obj_1.f == 1
    assert obj_2.f == 2


@pytest.mark.skipif(not PY_3_8_PLUS, reason="cached_property is 3.8+")
def test_slots_cached_properties_work_independently():
    """
    Multiple cached properties should work independently.
    """

    @attr.s(slots=True)
    class A:
        x = attr.ib()

        @functools.cached_property
        def f_1(self):
            return self.x

        @functools.cached_property
        def f_2(self):
            return self.x * 2

    obj = A(1)

    assert obj.f_1 == 1
    assert obj.f_2 == 2


@attr.s(slots=True)
class A:
    x = attr.ib()
    b = attr.ib()
    c = attr.ib()


def test_slots_unpickle_after_attr_removed():
    """
    We don't assign attributes we don't have anymore if the class has
    removed it.
    """
    a = A(1, 2, 3)
    a_pickled = pickle.dumps(a)
    a_unpickled = pickle.loads(a_pickled)
    assert a_unpickled == a

    @attr.s(slots=True)
    class NEW_A:
        x = attr.ib()
        c = attr.ib()

    with mock.patch(f"{__name__}.A", NEW_A):
        new_a = pickle.loads(a_pickled)

        assert new_a.x == 1
        assert new_a.c == 3
        assert not hasattr(new_a, "b")


def test_slots_unpickle_after_attr_added(frozen):
    """
    We don't assign attribute we haven't had before if the class has one added.
    """
    a = A(1, 2, 3)
    a_pickled = pickle.dumps(a)
    a_unpickled = pickle.loads(a_pickled)

    assert a_unpickled == a

    @attr.s(slots=True, frozen=frozen)
    class NEW_A:
        x = attr.ib()
        b = attr.ib()
        d = attr.ib()
        c = attr.ib()

    with mock.patch(f"{__name__}.A", NEW_A):
        new_a = pickle.loads(a_pickled)

        assert new_a.x == 1
        assert new_a.b == 2
        assert new_a.c == 3
        assert not hasattr(new_a, "d")


def test_slots_unpickle_is_backward_compatible(frozen):
    """
    Ensure object pickled before v22.2.0 can still be unpickled.
    """
    a = A(1, 2, 3)

    a_pickled = (
        b"\x80\x04\x95&\x00\x00\x00\x00\x00\x00\x00\x8c\x10"
        + a.__module__.encode()
        + b"\x94\x8c\x01A\x94\x93\x94)\x81\x94K\x01K\x02K\x03\x87\x94b."
    )

    a_unpickled = pickle.loads(a_pickled)

    assert a_unpickled == a
