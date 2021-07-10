"""
End-to-end tests.
"""

from __future__ import absolute_import, division, print_function

import pickle

import pytest
import six

from hypothesis import given
from hypothesis.strategies import booleans

import attr

from attr._compat import TYPE
from attr._make import NOTHING, Attribute
from attr.exceptions import FrozenInstanceError


@attr.s
class C1(object):
    x = attr.ib(validator=attr.validators.instance_of(int))
    y = attr.ib()


@attr.s(slots=True)
class C1Slots(object):
    x = attr.ib(validator=attr.validators.instance_of(int))
    y = attr.ib()


foo = None


@attr.s()
class C2(object):
    x = attr.ib(default=foo)
    y = attr.ib(default=attr.Factory(list))


@attr.s(slots=True)
class C2Slots(object):
    x = attr.ib(default=foo)
    y = attr.ib(default=attr.Factory(list))


@attr.s
class Super(object):
    x = attr.ib()

    def meth(self):
        return self.x


@attr.s(slots=True)
class SuperSlots(object):
    x = attr.ib()

    def meth(self):
        return self.x


@attr.s
class Sub(Super):
    y = attr.ib()


@attr.s(slots=True)
class SubSlots(SuperSlots):
    y = attr.ib()


@attr.s(frozen=True, slots=True)
class Frozen(object):
    x = attr.ib()


@attr.s
class SubFrozen(Frozen):
    y = attr.ib()


@attr.s(frozen=True, slots=False)
class FrozenNoSlots(object):
    x = attr.ib()


class Meta(type):
    pass


@attr.s
@six.add_metaclass(Meta)
class WithMeta(object):
    pass


@attr.s(slots=True)
@six.add_metaclass(Meta)
class WithMetaSlots(object):
    pass


FromMakeClass = attr.make_class("FromMakeClass", ["x"])


class TestDarkMagic(object):
    """
    Integration tests.
    """
    @pytest.mark.parametrize("cls", [C2, C2Slots])
    def test_fields(self, cls):
        """
        `attr.fields` works.
        """
        assert (
            Attribute(name="x", default=foo, validator=None,
                      repr=True, cmp=True, hash=None, init=True),
            Attribute(name="y", default=attr.Factory(list), validator=None,
                      repr=True, cmp=True, hash=None, init=True),
        ) == attr.fields(cls)

    @pytest.mark.parametrize("cls", [C1, C1Slots])
    def test_asdict(self, cls):
        """
        `attr.asdict` works.
        """
        assert {
            "x": 1,
            "y": 2,
        } == attr.asdict(cls(x=1, y=2))

    @pytest.mark.parametrize("cls", [C1, C1Slots])
    def test_validator(self, cls):
        """
        `instance_of` raises `TypeError` on type mismatch.
        """
        with pytest.raises(TypeError) as e:
            cls("1", 2)

        # Using C1 explicitly, since slot classes don't support this.
        assert (
            "'x' must be <{type} 'int'> (got '1' that is a <{type} "
            "'str'>).".format(type=TYPE),
            attr.fields(C1).x, int, "1",
        ) == e.value.args

    @given(booleans())
    def test_renaming(self, slots):
        """
        Private members are renamed but only in `__init__`.
        """
        @attr.s(slots=slots)
        class C3(object):
            _x = attr.ib()

        assert "C3(_x=1)" == repr(C3(x=1))

    @given(booleans(), booleans())
    def test_programmatic(self, slots, frozen):
        """
        `attr.make_class` works.
        """
        PC = attr.make_class("PC", ["a", "b"], slots=slots, frozen=frozen)
        assert (
            Attribute(name="a", default=NOTHING, validator=None,
                      repr=True, cmp=True, hash=None, init=True),
            Attribute(name="b", default=NOTHING, validator=None,
                      repr=True, cmp=True, hash=None, init=True),
        ) == attr.fields(PC)

    @pytest.mark.parametrize("cls", [Sub, SubSlots])
    def test_subclassing_with_extra_attrs(self, cls):
        """
        Sub-classing (where the subclass has extra attrs) does what you'd hope
        for.
        """
        obj = object()
        i = cls(x=obj, y=2)
        assert i.x is i.meth() is obj
        assert i.y == 2
        if cls is Sub:
            assert "Sub(x={obj}, y=2)".format(obj=obj) == repr(i)
        else:
            assert "SubSlots(x={obj}, y=2)".format(obj=obj) == repr(i)

    @pytest.mark.parametrize("base", [Super, SuperSlots])
    def test_subclass_without_extra_attrs(self, base):
        """
        Sub-classing (where the subclass does not have extra attrs) still
        behaves the same as a subclass with extra attrs.
        """
        class Sub2(base):
            pass

        obj = object()
        i = Sub2(x=obj)
        assert i.x is i.meth() is obj
        assert "Sub2(x={obj})".format(obj=obj) == repr(i)

    @pytest.mark.parametrize("frozen_class", [
        Frozen,  # has slots=True
        attr.make_class("FrozenToo", ["x"], slots=False, frozen=True),
    ])
    def test_frozen_instance(self, frozen_class):
        """
        Frozen instances can't be modified (easily).
        """
        frozen = frozen_class(1)

        with pytest.raises(FrozenInstanceError) as e:
            frozen.x = 2

        with pytest.raises(FrozenInstanceError) as e:
            del frozen.x

        assert e.value.args[0] == "can't set attribute"
        assert 1 == frozen.x

    @pytest.mark.parametrize("cls",
                             [C1, C1Slots, C2, C2Slots, Super, SuperSlots,
                              Sub, SubSlots, Frozen, FrozenNoSlots,
                              FromMakeClass])
    @pytest.mark.parametrize("protocol",
                             range(2, pickle.HIGHEST_PROTOCOL + 1))
    def test_pickle_attributes(self, cls, protocol):
        """
        Pickling/un-pickling of Attribute instances works.
        """
        for attribute in attr.fields(cls):
            assert attribute == pickle.loads(pickle.dumps(attribute, protocol))

    @pytest.mark.parametrize("cls",
                             [C1, C1Slots, C2, C2Slots, Super, SuperSlots,
                              Sub, SubSlots, Frozen, FrozenNoSlots,
                              FromMakeClass])
    @pytest.mark.parametrize("protocol",
                             range(2, pickle.HIGHEST_PROTOCOL + 1))
    def test_pickle_object(self, cls, protocol):
        """
        Pickle object serialization works on all kinds of attrs classes.
        """
        if len(attr.fields(cls)) == 2:
            obj = cls(123, 456)
        else:
            obj = cls(123)
        assert repr(obj) == repr(pickle.loads(pickle.dumps(obj, protocol)))

    def test_subclassing_frozen_gives_frozen(self):
        """
        The frozen-ness of classes is inherited.  Subclasses of frozen classes
        are also frozen and can be instantiated.
        """
        i = SubFrozen("foo", "bar")

        assert i.x == "foo"
        assert i.y == "bar"

    @pytest.mark.parametrize("cls", [WithMeta, WithMetaSlots])
    def test_metaclass_preserved(self, cls):
        """
        Metaclass data is preserved.
        """
        assert Meta == type(cls)

    def test_default_decorator(self):
        """
        Default decorator sets the default and the respective method gets
        called.
        """
        @attr.s
        class C(object):
            x = attr.ib(default=1)
            y = attr.ib()

            @y.default
            def compute(self):
                return self.x + 1

        assert C(1, 2) == C()

    @pytest.mark.parametrize("slots", [True, False])
    @pytest.mark.parametrize("frozen", [True, False])
    def test_attrib_overwrite(self, slots, frozen):
        """
        Subclasses can overwrite attributes of their superclass.
        """
        @attr.s(slots=slots, frozen=frozen)
        class SubOverwrite(Super):
            x = attr.ib(default=attr.Factory(list))

        assert SubOverwrite([]) == SubOverwrite()

    def test_dict_patch_class(self):
        """
        dict-classes are never replaced.
        """
        class C(object):
            x = attr.ib()

        C_new = attr.s(C)

        assert C_new is C

    def test_hash_by_id(self):
        """
        With dict classes, hashing by ID is active for hash=False even on
        Python 3.  This is incorrect behavior but we have to retain it for
        backward compatibility.
        """
        @attr.s(hash=False)
        class HashByIDBackwardCompat(object):
            x = attr.ib()

        assert (
            hash(HashByIDBackwardCompat(1)) != hash(HashByIDBackwardCompat(1))
        )

        @attr.s(hash=False, cmp=False)
        class HashByID(object):
            x = attr.ib()

        assert hash(HashByID(1)) != hash(HashByID(1))

        @attr.s(hash=True)
        class HashByValues(object):
            x = attr.ib()

        assert hash(HashByValues(1)) == hash(HashByValues(1))

    def test_handles_different_defaults(self):
        """
        Unhashable defaults + subclassing values work.
        """
        @attr.s
        class Unhashable(object):
            pass

        @attr.s
        class C(object):
            x = attr.ib(default=Unhashable())

        @attr.s
        class D(C):
            pass

    @pytest.mark.parametrize("slots", [True, False])
    def test_hash_false_cmp_false(self, slots):
        """
        hash=False and cmp=False make a class hashable by ID.
        """
        @attr.s(hash=False, cmp=False, slots=slots)
        class C(object):
            pass

        assert hash(C()) != hash(C())

    def test_overwrite_super(self):
        """
        Super classes can overwrite each other and the attributes are added
        in the order they are defined.
        """
        @attr.s
        class C(object):
            c = attr.ib(default=100)
            x = attr.ib(default=1)
            b = attr.ib(default=23)

        @attr.s
        class D(C):
            a = attr.ib(default=42)
            x = attr.ib(default=2)
            d = attr.ib(default=3.14)

        @attr.s
        class E(D):
            y = attr.ib(default=3)
            z = attr.ib(default=4)

        assert "E(c=100, b=23, a=42, x=2, d=3.14, y=3, z=4)" == repr(E())

    @pytest.mark.parametrize("base_slots", [True, False])
    @pytest.mark.parametrize("sub_slots", [True, False])
    @pytest.mark.parametrize("base_frozen", [True, False])
    @pytest.mark.parametrize("sub_frozen", [True, False])
    @pytest.mark.parametrize("base_converter", [True, False])
    @pytest.mark.parametrize("sub_converter", [True, False])
    def test_frozen_slots_combo(self, base_slots, sub_slots, base_frozen,
                                sub_frozen, base_converter, sub_converter):
        """
        A class with a single attribute, inheriting from another class
        with a single attribute.
        """

        @attr.s(frozen=base_frozen, slots=base_slots)
        class Base(object):
            a = attr.ib(converter=int if base_converter else None)

        @attr.s(frozen=sub_frozen, slots=sub_slots)
        class Sub(Base):
            b = attr.ib(converter=int if sub_converter else None)

        i = Sub("1", "2")

        assert i.a == (1 if base_converter else "1")
        assert i.b == (2 if sub_converter else "2")

        if base_frozen or sub_frozen:
            with pytest.raises(FrozenInstanceError):
                i.a = "2"

            with pytest.raises(FrozenInstanceError):
                i.b = "3"
