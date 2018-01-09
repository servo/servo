"""
Tests for PEP-526 type annotations.

Python 3.6+ only.
"""

import types
import typing

import pytest

import attr

from attr.exceptions import UnannotatedAttributeError


class TestAnnotations:
    """
    Tests for types derived from variable annotations (PEP-526).
    """

    def test_basic_annotations(self):
        """
        Sets the `Attribute.type` attr from basic type annotations.
        """
        @attr.s
        class C:
            x: int = attr.ib()
            y = attr.ib(type=str)
            z = attr.ib()

        assert int is attr.fields(C).x.type
        assert str is attr.fields(C).y.type
        assert None is attr.fields(C).z.type

    def test_catches_basic_type_conflict(self):
        """
        Raises ValueError if type is specified both ways.
        """
        with pytest.raises(ValueError) as e:
            @attr.s
            class C:
                x: int = attr.ib(type=int)

        assert (
            "Type annotation and type argument cannot both be present",
        ) == e.value.args

    def test_typing_annotations(self):
        """
        Sets the `Attribute.type` attr from typing annotations.
        """
        @attr.s
        class C:
            x: typing.List[int] = attr.ib()
            y = attr.ib(type=typing.Optional[str])

        assert typing.List[int] is attr.fields(C).x.type
        assert typing.Optional[str] is attr.fields(C).y.type

    def test_only_attrs_annotations_collected(self):
        """
        Annotations that aren't set to an attr.ib are ignored.
        """
        @attr.s
        class C:
            x: typing.List[int] = attr.ib()
            y: int

        assert 1 == len(attr.fields(C))

    @pytest.mark.parametrize("slots", [True, False])
    def test_auto_attribs(self, slots):
        """
        If *auto_attribs* is True, bare annotations are collected too.
        Defaults work and class variables are ignored.
        """
        @attr.s(auto_attribs=True, slots=slots)
        class C:
            cls_var: typing.ClassVar[int] = 23
            a: int
            x: typing.List[int] = attr.Factory(list)
            y: int = 2
            z: int = attr.ib(default=3)
            foo: typing.Any = None

        i = C(42)
        assert "C(a=42, x=[], y=2, z=3, foo=None)" == repr(i)

        attr_names = set(a.name for a in C.__attrs_attrs__)
        assert "a" in attr_names  # just double check that the set works
        assert "cls_var" not in attr_names

        assert int == attr.fields(C).a.type

        assert attr.Factory(list) == attr.fields(C).x.default
        assert typing.List[int] == attr.fields(C).x.type

        assert int == attr.fields(C).y.type
        assert 2 == attr.fields(C).y.default

        assert int == attr.fields(C).z.type

        assert typing.Any == attr.fields(C).foo.type

        # Class body is clean.
        if slots is False:
            with pytest.raises(AttributeError):
                C.y

            assert 2 == i.y
        else:
            assert isinstance(C.y, types.MemberDescriptorType)

            i.y = 23
            assert 23 == i.y

    @pytest.mark.parametrize("slots", [True, False])
    def test_auto_attribs_unannotated(self, slots):
        """
        Unannotated `attr.ib`s raise an error.
        """
        with pytest.raises(UnannotatedAttributeError) as e:
            @attr.s(slots=slots, auto_attribs=True)
            class C:
                v = attr.ib()
                x: int
                y = attr.ib()
                z: str

        assert (
            "The following `attr.ib`s lack a type annotation: v, y.",
        ) == e.value.args

    @pytest.mark.parametrize("slots", [True, False])
    def test_auto_attribs_subclassing(self, slots):
        """
        Attributes from super classes are inherited, it doesn't matter if the
        subclass has annotations or not.

        Ref #291
        """
        @attr.s(slots=slots, auto_attribs=True)
        class A:
            a: int = 1

        @attr.s(slots=slots, auto_attribs=True)
        class B(A):
            b: int = 2

        @attr.s(slots=slots, auto_attribs=True)
        class C(A):
            pass

        assert "B(a=1, b=2)" == repr(B())
        assert "C(a=1)" == repr(C())
