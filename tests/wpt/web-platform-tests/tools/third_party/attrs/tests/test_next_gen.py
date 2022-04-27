# SPDX-License-Identifier: MIT

"""
Python 3-only integration tests for provisional next generation APIs.
"""

import re

from functools import partial

import pytest

import attr as _attr  # don't use it by accident
import attrs


@attrs.define
class C:
    x: str
    y: int


class TestNextGen:
    def test_simple(self):
        """
        Instantiation works.
        """
        C("1", 2)

    def test_no_slots(self):
        """
        slots can be deactivated.
        """

        @attrs.define(slots=False)
        class NoSlots:
            x: int

        ns = NoSlots(1)

        assert {"x": 1} == getattr(ns, "__dict__")

    def test_validates(self):
        """
        Validators at __init__ and __setattr__ work.
        """

        @attrs.define
        class Validated:
            x: int = attrs.field(validator=attrs.validators.instance_of(int))

        v = Validated(1)

        with pytest.raises(TypeError):
            Validated(None)

        with pytest.raises(TypeError):
            v.x = "1"

    def test_no_order(self):
        """
        Order is off by default but can be added.
        """
        with pytest.raises(TypeError):
            C("1", 2) < C("2", 3)

        @attrs.define(order=True)
        class Ordered:
            x: int

        assert Ordered(1) < Ordered(2)

    def test_override_auto_attribs_true(self):
        """
        Don't guess if auto_attrib is set explicitly.

        Having an unannotated attrs.ib/attrs.field fails.
        """
        with pytest.raises(attrs.exceptions.UnannotatedAttributeError):

            @attrs.define(auto_attribs=True)
            class ThisFails:
                x = attrs.field()
                y: int

    def test_override_auto_attribs_false(self):
        """
        Don't guess if auto_attrib is set explicitly.

        Annotated fields that don't carry an attrs.ib are ignored.
        """

        @attrs.define(auto_attribs=False)
        class NoFields:
            x: int
            y: int

        assert NoFields() == NoFields()

    def test_auto_attribs_detect(self):
        """
        define correctly detects if a class lacks type annotations.
        """

        @attrs.define
        class OldSchool:
            x = attrs.field()

        assert OldSchool(1) == OldSchool(1)

        # Test with maybe_cls = None
        @attrs.define()
        class OldSchool2:
            x = attrs.field()

        assert OldSchool2(1) == OldSchool2(1)

    def test_auto_attribs_detect_fields_and_annotations(self):
        """
        define infers auto_attribs=True if fields have type annotations
        """

        @attrs.define
        class NewSchool:
            x: int
            y: list = attrs.field()

            @y.validator
            def _validate_y(self, attribute, value):
                if value < 0:
                    raise ValueError("y must be positive")

        assert NewSchool(1, 1) == NewSchool(1, 1)
        with pytest.raises(ValueError):
            NewSchool(1, -1)
        assert list(attrs.fields_dict(NewSchool).keys()) == ["x", "y"]

    def test_auto_attribs_partially_annotated(self):
        """
        define infers auto_attribs=True if any type annotations are found
        """

        @attrs.define
        class NewSchool:
            x: int
            y: list
            z = 10

        # fields are defined for any annotated attributes
        assert NewSchool(1, []) == NewSchool(1, [])
        assert list(attrs.fields_dict(NewSchool).keys()) == ["x", "y"]

        # while the unannotated attributes are left as class vars
        assert NewSchool.z == 10
        assert "z" in NewSchool.__dict__

    def test_auto_attribs_detect_annotations(self):
        """
        define correctly detects if a class has type annotations.
        """

        @attrs.define
        class NewSchool:
            x: int

        assert NewSchool(1) == NewSchool(1)

        # Test with maybe_cls = None
        @attrs.define()
        class NewSchool2:
            x: int

        assert NewSchool2(1) == NewSchool2(1)

    def test_exception(self):
        """
        Exceptions are detected and correctly handled.
        """

        @attrs.define
        class E(Exception):
            msg: str
            other: int

        with pytest.raises(E) as ei:
            raise E("yolo", 42)

        e = ei.value

        assert ("yolo", 42) == e.args
        assert "yolo" == e.msg
        assert 42 == e.other

    def test_frozen(self):
        """
        attrs.frozen freezes classes.
        """

        @attrs.frozen
        class F:
            x: str

        f = F(1)

        with pytest.raises(attrs.exceptions.FrozenInstanceError):
            f.x = 2

    def test_auto_detect_eq(self):
        """
        auto_detect=True works for eq.

        Regression test for #670.
        """

        @attrs.define
        class C:
            def __eq__(self, o):
                raise ValueError()

        with pytest.raises(ValueError):
            C() == C()

    def test_subclass_frozen(self):
        """
        It's possible to subclass an `attrs.frozen` class and the frozen-ness
        is inherited.
        """

        @attrs.frozen
        class A:
            a: int

        @attrs.frozen
        class B(A):
            b: int

        @attrs.define(on_setattr=attrs.setters.NO_OP)
        class C(B):
            c: int

        assert B(1, 2) == B(1, 2)
        assert C(1, 2, 3) == C(1, 2, 3)

        with pytest.raises(attrs.exceptions.FrozenInstanceError):
            A(1).a = 1

        with pytest.raises(attrs.exceptions.FrozenInstanceError):
            B(1, 2).a = 1

        with pytest.raises(attrs.exceptions.FrozenInstanceError):
            B(1, 2).b = 2

        with pytest.raises(attrs.exceptions.FrozenInstanceError):
            C(1, 2, 3).c = 3

    def test_catches_frozen_on_setattr(self):
        """
        Passing frozen=True and on_setattr hooks is caught, even if the
        immutability is inherited.
        """

        @attrs.define(frozen=True)
        class A:
            pass

        with pytest.raises(
            ValueError, match="Frozen classes can't use on_setattr."
        ):

            @attrs.define(frozen=True, on_setattr=attrs.setters.validate)
            class B:
                pass

        with pytest.raises(
            ValueError,
            match=re.escape(
                "Frozen classes can't use on_setattr "
                "(frozen-ness was inherited)."
            ),
        ):

            @attrs.define(on_setattr=attrs.setters.validate)
            class C(A):
                pass

    @pytest.mark.parametrize(
        "decorator",
        [
            partial(_attr.s, frozen=True, slots=True, auto_exc=True),
            attrs.frozen,
            attrs.define,
            attrs.mutable,
        ],
    )
    def test_discard_context(self, decorator):
        """
        raise from None works.

        Regression test for #703.
        """

        @decorator
        class MyException(Exception):
            x: str = attrs.field()

        with pytest.raises(MyException) as ei:
            try:
                raise ValueError()
            except ValueError:
                raise MyException("foo") from None

        assert "foo" == ei.value.x
        assert ei.value.__cause__ is None

    def test_converts_and_validates_by_default(self):
        """
        If no on_setattr is set, assume setters.convert, setters.validate.
        """

        @attrs.define
        class C:
            x: int = attrs.field(converter=int)

            @x.validator
            def _v(self, _, value):
                if value < 10:
                    raise ValueError("must be >=10")

        inst = C(10)

        # Converts
        inst.x = "11"

        assert 11 == inst.x

        # Validates
        with pytest.raises(ValueError, match="must be >=10"):
            inst.x = "9"

    def test_mro_ng(self):
        """
        Attributes and methods are looked up the same way in NG by default.

        See #428
        """

        @attrs.define
        class A:

            x: int = 10

            def xx(self):
                return 10

        @attrs.define
        class B(A):
            y: int = 20

        @attrs.define
        class C(A):
            x: int = 50

            def xx(self):
                return 50

        @attrs.define
        class D(B, C):
            pass

        d = D()

        assert d.x == d.xx()


class TestAsTuple:
    def test_smoke(self):
        """
        `attrs.astuple` only changes defaults, so we just call it and compare.
        """
        inst = C("foo", 42)

        assert attrs.astuple(inst) == _attr.astuple(inst)


class TestAsDict:
    def test_smoke(self):
        """
        `attrs.asdict` only changes defaults, so we just call it and compare.
        """
        inst = C("foo", {(1,): 42})

        assert attrs.asdict(inst) == _attr.asdict(
            inst, retain_collection_types=True
        )


class TestImports:
    """
    Verify our re-imports and mirroring works.
    """

    def test_converters(self):
        """
        Importing from attrs.converters works.
        """
        from attrs.converters import optional

        assert optional is _attr.converters.optional

    def test_exceptions(self):
        """
        Importing from attrs.exceptions works.
        """
        from attrs.exceptions import FrozenError

        assert FrozenError is _attr.exceptions.FrozenError

    def test_filters(self):
        """
        Importing from attrs.filters works.
        """
        from attrs.filters import include

        assert include is _attr.filters.include

    def test_setters(self):
        """
        Importing from attrs.setters works.
        """
        from attrs.setters import pipe

        assert pipe is _attr.setters.pipe

    def test_validators(self):
        """
        Importing from attrs.validators works.
        """
        from attrs.validators import and_

        assert and_ is _attr.validators.and_
