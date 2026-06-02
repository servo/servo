# SPDX-License-Identifier: MIT

"""
Tests for `attr._make`.
"""


import copy
import functools
import gc
import inspect
import itertools
import sys

from operator import attrgetter
from typing import Generic, TypeVar

import pytest

from hypothesis import assume, given
from hypothesis.strategies import booleans, integers, lists, sampled_from, text

import attr

from attr import _config
from attr._compat import PY310
from attr._make import (
    Attribute,
    Factory,
    _AndValidator,
    _Attributes,
    _ClassBuilder,
    _CountingAttr,
    _determine_attrib_eq_order,
    _determine_attrs_eq_order,
    _determine_whether_to_implement,
    _transform_attrs,
    and_,
    fields,
    fields_dict,
    make_class,
    validate,
)
from attr.exceptions import DefaultAlreadySetError, NotAnAttrsClassError

from .strategies import (
    gen_attr_names,
    list_of_attrs,
    optional_bool,
    simple_attrs,
    simple_attrs_with_metadata,
    simple_attrs_without_metadata,
    simple_classes,
)
from .utils import simple_attr


attrs_st = simple_attrs.map(lambda c: Attribute.from_counting_attr("name", c))


@pytest.fixture(name="with_and_without_validation", params=[True, False])
def _with_and_without_validation(request):
    """
    Run tests with and without validation enabled.
    """
    attr.validators.set_disabled(request.param)

    try:
        yield
    finally:
        attr.validators.set_disabled(False)


class TestCountingAttr:
    """
    Tests for `attr`.
    """

    def test_returns_Attr(self):
        """
        Returns an instance of _CountingAttr.
        """
        a = attr.ib()

        assert isinstance(a, _CountingAttr)

    def test_validators_lists_to_wrapped_tuples(self):
        """
        If a list is passed as validator, it's just converted to a tuple.
        """

        def v1(_, __):
            pass

        def v2(_, __):
            pass

        a = attr.ib(validator=[v1, v2])

        assert _AndValidator((v1, v2)) == a._validator

    def test_validator_decorator_single(self):
        """
        If _CountingAttr.validator is used as a decorator and there is no
        decorator set, the decorated method is used as the validator.
        """
        a = attr.ib()

        @a.validator
        def v():
            pass

        assert v == a._validator

    @pytest.mark.parametrize(
        "wrap", [lambda v: v, lambda v: [v], lambda v: and_(v)]
    )
    def test_validator_decorator(self, wrap):
        """
        If _CountingAttr.validator is used as a decorator and there is already
        a decorator set, the decorators are composed using `and_`.
        """

        def v(_, __):
            pass

        a = attr.ib(validator=wrap(v))

        @a.validator
        def v2(self, _, __):
            pass

        assert _AndValidator((v, v2)) == a._validator

    def test_default_decorator_already_set(self):
        """
        Raise DefaultAlreadySetError if the decorator is used after a default
        has been set.
        """
        a = attr.ib(default=42)

        with pytest.raises(DefaultAlreadySetError):

            @a.default
            def f(self):
                pass

    def test_default_decorator_sets(self):
        """
        Decorator wraps the method in a Factory with pass_self=True and sets
        the default.
        """
        a = attr.ib()

        @a.default
        def f(self):
            pass

        assert Factory(f, True) == a._default


def make_tc():
    class TransformC:
        z = attr.ib()
        y = attr.ib()
        x = attr.ib()
        a = 42

    return TransformC


class TestTransformAttrs:
    """
    Tests for `_transform_attrs`.
    """

    def test_no_modifications(self):
        """
        Does not attach __attrs_attrs__ to the class.
        """
        C = make_tc()
        _transform_attrs(C, None, False, False, True, None)

        assert None is getattr(C, "__attrs_attrs__", None)

    def test_normal(self):
        """
        Transforms every `_CountingAttr` and leaves others (a) be.
        """
        C = make_tc()
        attrs, _, _ = _transform_attrs(C, None, False, False, True, None)

        assert ["z", "y", "x"] == [a.name for a in attrs]

    def test_empty(self):
        """
        No attributes works as expected.
        """

        @attr.s
        class C:
            pass

        assert _Attributes(((), [], {})) == _transform_attrs(
            C, None, False, False, True, None
        )

    def test_transforms_to_attribute(self):
        """
        All `_CountingAttr`s are transformed into `Attribute`s.
        """
        C = make_tc()
        attrs, base_attrs, _ = _transform_attrs(
            C, None, False, False, True, None
        )

        assert [] == base_attrs
        assert 3 == len(attrs)
        assert all(isinstance(a, Attribute) for a in attrs)

    def test_conflicting_defaults(self):
        """
        Raises `ValueError` if attributes with defaults are followed by
        mandatory attributes.
        """

        class C:
            x = attr.ib(default=None)
            y = attr.ib()

        with pytest.raises(ValueError) as e:
            _transform_attrs(C, None, False, False, True, None)
        assert (
            "No mandatory attributes allowed after an attribute with a "
            "default value or factory.  Attribute in question: Attribute"
            "(name='y', default=NOTHING, validator=None, repr=True, "
            "eq=True, eq_key=None, order=True, order_key=None, "
            "hash=None, init=True, "
            "metadata=mappingproxy({}), type=None, converter=None, "
            "kw_only=False, inherited=False, on_setattr=None, alias=None)",
        ) == e.value.args

    def test_kw_only(self):
        """
        Converts all attributes, including base class' attributes, if `kw_only`
        is provided. Therefore, `kw_only` allows attributes with defaults to
        precede mandatory attributes.

        Updates in the subclass *don't* affect the base class attributes.
        """

        @attr.s
        class B:
            b = attr.ib()

        for b_a in B.__attrs_attrs__:
            assert b_a.kw_only is False

        class C(B):
            x = attr.ib(default=None)
            y = attr.ib()

        attrs, base_attrs, _ = _transform_attrs(
            C, None, False, True, True, None
        )

        assert len(attrs) == 3
        assert len(base_attrs) == 1

        for a in attrs:
            assert a.kw_only is True

        for b_a in B.__attrs_attrs__:
            assert b_a.kw_only is False

    def test_these(self):
        """
        If these is passed, use it and ignore body and base classes.
        """

        class Base:
            z = attr.ib()

        class C(Base):
            y = attr.ib()

        attrs, base_attrs, _ = _transform_attrs(
            C, {"x": attr.ib()}, False, False, True, None
        )

        assert [] == base_attrs
        assert (simple_attr("x"),) == attrs

    def test_these_leave_body(self):
        """
        If these is passed, no attributes are removed from the body.
        """

        @attr.s(init=False, these={"x": attr.ib()})
        class C:
            x = 5

        assert 5 == C().x
        assert "C(x=5)" == repr(C())

    def test_these_ordered(self):
        """
        If these is passed ordered attrs, their order respect instead of the
        counter.
        """
        b = attr.ib(default=2)
        a = attr.ib(default=1)

        @attr.s(these={"a": a, "b": b})
        class C:
            pass

        assert "C(a=1, b=2)" == repr(C())

    def test_multiple_inheritance_old(self):
        """
        Old multiple inheritance attribute collection behavior is retained.

        See #285
        """

        @attr.s
        class A:
            a1 = attr.ib(default="a1")
            a2 = attr.ib(default="a2")

        @attr.s
        class B(A):
            b1 = attr.ib(default="b1")
            b2 = attr.ib(default="b2")

        @attr.s
        class C(B, A):
            c1 = attr.ib(default="c1")
            c2 = attr.ib(default="c2")

        @attr.s
        class D(A):
            d1 = attr.ib(default="d1")
            d2 = attr.ib(default="d2")

        @attr.s
        class E(C, D):
            e1 = attr.ib(default="e1")
            e2 = attr.ib(default="e2")

        assert (
            "E(a1='a1', a2='a2', b1='b1', b2='b2', c1='c1', c2='c2', d1='d1', "
            "d2='d2', e1='e1', e2='e2')"
        ) == repr(E())

    def test_overwrite_proper_mro(self):
        """
        The proper MRO path works single overwrites too.
        """

        @attr.s(collect_by_mro=True)
        class C:
            x = attr.ib(default=1)

        @attr.s(collect_by_mro=True)
        class D(C):
            x = attr.ib(default=2)

        assert "D(x=2)" == repr(D())

    def test_multiple_inheritance_proper_mro(self):
        """
        Attributes are collected according to the MRO.

        See #428
        """

        @attr.s
        class A:
            a1 = attr.ib(default="a1")
            a2 = attr.ib(default="a2")

        @attr.s
        class B(A):
            b1 = attr.ib(default="b1")
            b2 = attr.ib(default="b2")

        @attr.s
        class C(B, A):
            c1 = attr.ib(default="c1")
            c2 = attr.ib(default="c2")

        @attr.s
        class D(A):
            d1 = attr.ib(default="d1")
            d2 = attr.ib(default="d2")

        @attr.s(collect_by_mro=True)
        class E(C, D):
            e1 = attr.ib(default="e1")
            e2 = attr.ib(default="e2")

        assert (
            "E(a1='a1', a2='a2', d1='d1', d2='d2', b1='b1', b2='b2', c1='c1', "
            "c2='c2', e1='e1', e2='e2')"
        ) == repr(E())

    def test_mro(self):
        """
        Attributes and methods are looked up the same way.

        See #428
        """

        @attr.s(collect_by_mro=True)
        class A:
            x = attr.ib(10)

            def xx(self):
                return 10

        @attr.s(collect_by_mro=True)
        class B(A):
            y = attr.ib(20)

        @attr.s(collect_by_mro=True)
        class C(A):
            x = attr.ib(50)

            def xx(self):
                return 50

        @attr.s(collect_by_mro=True)
        class D(B, C):
            pass

        d = D()

        assert d.x == d.xx()

    def test_inherited(self):
        """
        Inherited Attributes have `.inherited` True, otherwise False.
        """

        @attr.s
        class A:
            a = attr.ib()

        @attr.s
        class B(A):
            b = attr.ib()

        @attr.s
        class C(B):
            a = attr.ib()
            c = attr.ib()

        f = attr.fields

        assert False is f(A).a.inherited

        assert True is f(B).a.inherited
        assert False is f(B).b.inherited

        assert False is f(C).a.inherited
        assert True is f(C).b.inherited
        assert False is f(C).c.inherited


class TestAttributes:
    """
    Tests for the `attrs`/`attr.s` class decorator.
    """

    def test_sets_attrs(self):
        """
        Sets the `__attrs_attrs__` class attribute with a list of `Attribute`s.
        """

        @attr.s
        class C:
            x = attr.ib()

        assert "x" == C.__attrs_attrs__[0].name
        assert all(isinstance(a, Attribute) for a in C.__attrs_attrs__)

    def test_empty(self):
        """
        No attributes, no problems.
        """

        @attr.s
        class C3:
            pass

        assert "C3()" == repr(C3())
        assert C3() == C3()

    @given(attr=attrs_st, attr_name=sampled_from(Attribute.__slots__))
    def test_immutable(self, attr, attr_name):
        """
        Attribute instances are immutable.
        """
        with pytest.raises(AttributeError):
            setattr(attr, attr_name, 1)

    @pytest.mark.parametrize(
        "method_name", ["__repr__", "__eq__", "__hash__", "__init__"]
    )
    def test_adds_all_by_default(self, method_name):
        """
        If no further arguments are supplied, all add_XXX functions except
        add_hash are applied.  __hash__ is set to None.
        """
        # Set the method name to a sentinel and check whether it has been
        # overwritten afterwards.
        sentinel = object()

        class C:
            x = attr.ib()

        setattr(C, method_name, sentinel)

        C = attr.s(C)
        meth = getattr(C, method_name)

        assert sentinel != meth
        if method_name == "__hash__":
            assert meth is None

    @pytest.mark.parametrize(
        ("arg_name", "method_name"),
        [
            ("repr", "__repr__"),
            ("eq", "__eq__"),
            ("order", "__le__"),
            ("hash", "__hash__"),
            ("init", "__init__"),
        ],
    )
    def test_respects_add_arguments(self, arg_name, method_name):
        """
        If a certain `XXX` is `False`, `__XXX__` is not added to the class.
        """
        # Set the method name to a sentinel and check whether it has been
        # overwritten afterwards.
        sentinel = object()

        am_args = {
            "repr": True,
            "eq": True,
            "order": True,
            "hash": True,
            "init": True,
        }
        am_args[arg_name] = False
        if arg_name == "eq":
            am_args["order"] = False

        class C:
            x = attr.ib()

        setattr(C, method_name, sentinel)

        C = attr.s(**am_args)(C)

        assert sentinel == getattr(C, method_name)

    @pytest.mark.parametrize("init", [True, False])
    def test_respects_init_attrs_init(self, init):
        """
        If init=False, adds __attrs_init__ to the class.
        Otherwise, it does not.
        """

        class C:
            x = attr.ib()

        C = attr.s(init=init)(C)
        assert hasattr(C, "__attrs_init__") != init

    @given(slots_outer=booleans(), slots_inner=booleans())
    def test_repr_qualname(self, slots_outer, slots_inner):
        """
        The name in repr is the __qualname__.
        """

        @attr.s(slots=slots_outer)
        class C:
            @attr.s(slots=slots_inner)
            class D:
                pass

        assert "C.D()" == repr(C.D())
        assert "GC.D()" == repr(GC.D())

    @given(slots_outer=booleans(), slots_inner=booleans())
    def test_repr_fake_qualname(self, slots_outer, slots_inner):
        """
        Setting repr_ns overrides a potentially guessed namespace.
        """

        @attr.s(slots=slots_outer)
        class C:
            @attr.s(repr_ns="C", slots=slots_inner)
            class D:
                pass

        assert "C.D()" == repr(C.D())

    @given(slots_outer=booleans(), slots_inner=booleans())
    def test_name_not_overridden(self, slots_outer, slots_inner):
        """
        __name__ is different from __qualname__.
        """

        @attr.s(slots=slots_outer)
        class C:
            @attr.s(slots=slots_inner)
            class D:
                pass

        assert C.D.__name__ == "D"
        assert C.D.__qualname__ == C.__qualname__ + ".D"

    @pytest.mark.usefixtures("with_and_without_validation")
    def test_pre_init(self):
        """
        Verify that __attrs_pre_init__ gets called if defined.
        """

        @attr.s
        class C:
            def __attrs_pre_init__(self2):
                self2.z = 30

        c = C()

        assert 30 == getattr(c, "z", None)

    @pytest.mark.usefixtures("with_and_without_validation")
    def test_pre_init_args(self):
        """
        Verify that __attrs_pre_init__ gets called with extra args if defined.
        """

        @attr.s
        class C:
            x = attr.ib()

            def __attrs_pre_init__(self2, x):
                self2.z = x + 1

        c = C(x=10)

        assert 11 == getattr(c, "z", None)

    @pytest.mark.usefixtures("with_and_without_validation")
    def test_pre_init_kwargs(self):
        """
        Verify that __attrs_pre_init__ gets called with extra args and kwargs
        if defined.
        """

        @attr.s
        class C:
            x = attr.ib()
            y = attr.field(kw_only=True)

            def __attrs_pre_init__(self2, x, y):
                self2.z = x + y + 1

        c = C(10, y=11)

        assert 22 == getattr(c, "z", None)

    @pytest.mark.usefixtures("with_and_without_validation")
    def test_pre_init_kwargs_only(self):
        """
        Verify that __attrs_pre_init__ gets called with extra kwargs only if
        defined.
        """

        @attr.s
        class C:
            y = attr.field(kw_only=True)

            def __attrs_pre_init__(self2, y):
                self2.z = y + 1

        c = C(y=11)

        assert 12 == getattr(c, "z", None)

    @pytest.mark.usefixtures("with_and_without_validation")
    def test_post_init(self):
        """
        Verify that __attrs_post_init__ gets called if defined.
        """

        @attr.s
        class C:
            x = attr.ib()
            y = attr.ib()

            def __attrs_post_init__(self2):
                self2.z = self2.x + self2.y

        c = C(x=10, y=20)

        assert 30 == getattr(c, "z", None)

    @pytest.mark.usefixtures("with_and_without_validation")
    def test_pre_post_init_order(self):
        """
        Verify that __attrs_post_init__ gets called if defined.
        """

        @attr.s
        class C:
            x = attr.ib()

            def __attrs_pre_init__(self2):
                self2.z = 30

            def __attrs_post_init__(self2):
                self2.z += self2.x

        c = C(x=10)

        assert 40 == getattr(c, "z", None)

    def test_types(self):
        """
        Sets the `Attribute.type` attr from type argument.
        """

        @attr.s
        class C:
            x = attr.ib(type=int)
            y = attr.ib(type=str)
            z = attr.ib()

        assert int is fields(C).x.type
        assert str is fields(C).y.type
        assert None is fields(C).z.type

    def test_clean_class(self, slots):
        """
        Attribute definitions do not appear on the class body after @attr.s.
        """

        @attr.s(slots=slots)
        class C:
            x = attr.ib()

        x = getattr(C, "x", None)

        assert not isinstance(x, _CountingAttr)

    def test_factory_sugar(self):
        """
        Passing factory=f is syntactic sugar for passing default=Factory(f).
        """

        @attr.s
        class C:
            x = attr.ib(factory=list)

        assert Factory(list) == attr.fields(C).x.default

    def test_sugar_factory_mutex(self):
        """
        Passing both default and factory raises ValueError.
        """
        with pytest.raises(ValueError, match="mutually exclusive"):

            @attr.s
            class C:
                x = attr.ib(factory=list, default=Factory(list))

    def test_sugar_callable(self):
        """
        Factory has to be a callable to prevent people from passing Factory
        into it.
        """
        with pytest.raises(ValueError, match="must be a callable"):

            @attr.s
            class C:
                x = attr.ib(factory=Factory(list))

    def test_inherited_does_not_affect_hashing_and_equality(self):
        """
        Whether or not an Attribute has been inherited doesn't affect how it's
        hashed and compared.
        """

        @attr.s
        class BaseClass:
            x = attr.ib()

        @attr.s
        class SubClass(BaseClass):
            pass

        ba = attr.fields(BaseClass)[0]
        sa = attr.fields(SubClass)[0]

        assert ba == sa
        assert hash(ba) == hash(sa)


class TestKeywordOnlyAttributes:
    """
    Tests for keyword-only attributes.
    """

    def test_adds_keyword_only_arguments(self):
        """
        Attributes can be added as keyword-only.
        """

        @attr.s
        class C:
            a = attr.ib()
            b = attr.ib(default=2, kw_only=True)
            c = attr.ib(kw_only=True)
            d = attr.ib(default=attr.Factory(lambda: 4), kw_only=True)

        c = C(1, c=3)

        assert c.a == 1
        assert c.b == 2
        assert c.c == 3
        assert c.d == 4

    def test_ignores_kw_only_when_init_is_false(self):
        """
        Specifying ``kw_only=True`` when ``init=False`` is essentially a no-op.
        """

        @attr.s
        class C:
            x = attr.ib(init=False, default=0, kw_only=True)
            y = attr.ib()

        c = C(1)

        assert c.x == 0
        assert c.y == 1

    def test_keyword_only_attributes_presence(self):
        """
        Raises `TypeError` when keyword-only arguments are
        not specified.
        """

        @attr.s
        class C:
            x = attr.ib(kw_only=True)

        with pytest.raises(TypeError) as e:
            C()

        assert (
            "missing 1 required keyword-only argument: 'x'"
        ) in e.value.args[0]

    def test_keyword_only_attributes_unexpected(self):
        """
        Raises `TypeError` when unexpected keyword argument passed.
        """

        @attr.s
        class C:
            x = attr.ib(kw_only=True)

        with pytest.raises(TypeError) as e:
            C(x=5, y=10)

        assert "got an unexpected keyword argument 'y'" in e.value.args[0]

    def test_keyword_only_attributes_can_come_in_any_order(self):
        """
        Mandatory vs non-mandatory attr order only matters when they are part
        of the __init__ signature and when they aren't kw_only (which are
        moved to the end and can be mandatory or non-mandatory in any order,
        as they will be specified as keyword args anyway).
        """

        @attr.s
        class C:
            a = attr.ib(kw_only=True)
            b = attr.ib(kw_only=True, default="b")
            c = attr.ib(kw_only=True)
            d = attr.ib()
            e = attr.ib(default="e")
            f = attr.ib(kw_only=True)
            g = attr.ib(kw_only=True, default="g")
            h = attr.ib(kw_only=True)
            i = attr.ib(init=False)

        c = C("d", a="a", c="c", f="f", h="h")

        assert c.a == "a"
        assert c.b == "b"
        assert c.c == "c"
        assert c.d == "d"
        assert c.e == "e"
        assert c.f == "f"
        assert c.g == "g"
        assert c.h == "h"

    def test_keyword_only_attributes_allow_subclassing(self):
        """
        Subclass can define keyword-only attributed without defaults,
        when the base class has attributes with defaults.
        """

        @attr.s
        class Base:
            x = attr.ib(default=0)

        @attr.s
        class C(Base):
            y = attr.ib(kw_only=True)

        c = C(y=1)

        assert c.x == 0
        assert c.y == 1

    def test_keyword_only_class_level(self):
        """
        `kw_only` can be provided at the attr.s level, converting all
        attributes to `kw_only.`
        """

        @attr.s(kw_only=True)
        class C:
            x = attr.ib()
            y = attr.ib(kw_only=True)

        with pytest.raises(TypeError):
            C(0, y=1)

        c = C(x=0, y=1)

        assert c.x == 0
        assert c.y == 1

    def test_keyword_only_class_level_subclassing(self):
        """
        Subclass `kw_only` propagates to attrs inherited from the base,
        allowing non-default following default.
        """

        @attr.s
        class Base:
            x = attr.ib(default=0)

        @attr.s(kw_only=True)
        class C(Base):
            y = attr.ib()

        with pytest.raises(TypeError):
            C(1)

        c = C(x=0, y=1)

        assert c.x == 0
        assert c.y == 1

    def test_init_false_attribute_after_keyword_attribute(self):
        """
        A positional attribute cannot follow a `kw_only` attribute,
        but an `init=False` attribute can because it won't appear
        in `__init__`
        """

        @attr.s
        class KwArgBeforeInitFalse:
            kwarg = attr.ib(kw_only=True)
            non_init_function_default = attr.ib(init=False)
            non_init_keyword_default = attr.ib(
                init=False, default="default-by-keyword"
            )

            @non_init_function_default.default
            def _init_to_init(self):
                return self.kwarg + "b"

        c = KwArgBeforeInitFalse(kwarg="a")

        assert c.kwarg == "a"
        assert c.non_init_function_default == "ab"
        assert c.non_init_keyword_default == "default-by-keyword"

    def test_init_false_attribute_after_keyword_attribute_with_inheritance(
        self,
    ):
        """
        A positional attribute cannot follow a `kw_only` attribute,
        but an `init=False` attribute can because it won't appear
        in `__init__`. This test checks that we allow this
        even when the `kw_only` attribute appears in a parent class
        """

        @attr.s
        class KwArgBeforeInitFalseParent:
            kwarg = attr.ib(kw_only=True)

        @attr.s
        class KwArgBeforeInitFalseChild(KwArgBeforeInitFalseParent):
            non_init_function_default = attr.ib(init=False)
            non_init_keyword_default = attr.ib(
                init=False, default="default-by-keyword"
            )

            @non_init_function_default.default
            def _init_to_init(self):
                return self.kwarg + "b"

        c = KwArgBeforeInitFalseChild(kwarg="a")

        assert c.kwarg == "a"
        assert c.non_init_function_default == "ab"
        assert c.non_init_keyword_default == "default-by-keyword"


@attr.s
class GC:
    @attr.s
    class D:
        pass


class TestMakeClass:
    """
    Tests for `make_class`.
    """

    @pytest.mark.parametrize("ls", [list, tuple])
    def test_simple(self, ls):
        """
        Passing a list of strings creates attributes with default args.
        """
        C1 = make_class("C1", ls(["a", "b"]))

        @attr.s
        class C2:
            a = attr.ib()
            b = attr.ib()

        assert C1.__attrs_attrs__ == C2.__attrs_attrs__

    def test_dict(self):
        """
        Passing a dict of name: _CountingAttr creates an equivalent class.
        """
        C1 = make_class(
            "C1", {"a": attr.ib(default=42), "b": attr.ib(default=None)}
        )

        @attr.s
        class C2:
            a = attr.ib(default=42)
            b = attr.ib(default=None)

        assert C1.__attrs_attrs__ == C2.__attrs_attrs__

    def test_attr_args(self):
        """
        attributes_arguments are passed to attributes
        """
        C = make_class("C", ["x"], repr=False)

        assert repr(C(1)).startswith("<tests.test_make.C object at 0x")

    def test_catches_wrong_attrs_type(self):
        """
        Raise `TypeError` if an invalid type for attrs is passed.
        """
        with pytest.raises(TypeError) as e:
            make_class("C", object())

        assert ("attrs argument must be a dict or a list.",) == e.value.args

    def test_bases(self):
        """
        Parameter bases default to (object,) and subclasses correctly
        """

        class D:
            pass

        cls = make_class("C", {})

        assert cls.__mro__[-1] == object

        cls = make_class("C", {}, bases=(D,))

        assert D in cls.__mro__
        assert isinstance(cls(), D)

    def test_additional_class_body(self):
        """
        Additional class_body is added to newly created class.
        """

        def echo_func(cls, *args):
            return args

        cls = make_class("C", {}, class_body={"echo": classmethod(echo_func)})

        assert ("a", "b") == cls.echo("a", "b")

    def test_clean_class(self, slots):
        """
        Attribute definitions do not appear on the class body.
        """
        C = make_class("C", ["x"], slots=slots)

        x = getattr(C, "x", None)

        assert not isinstance(x, _CountingAttr)

    def test_missing_sys_getframe(self, monkeypatch):
        """
        `make_class()` does not fail when `sys._getframe()` is not available.
        """
        monkeypatch.delattr(sys, "_getframe")
        C = make_class("C", ["x"])

        assert 1 == len(C.__attrs_attrs__)

    def test_make_class_ordered(self):
        """
        If `make_class()` is passed ordered attrs, their order is respected
        instead of the counter.
        """
        b = attr.ib(default=2)
        a = attr.ib(default=1)

        C = attr.make_class("C", {"a": a, "b": b})

        assert "C(a=1, b=2)" == repr(C())

    def test_generic_dynamic_class(self):
        """
        make_class can create generic dynamic classes.

        https://github.com/python-attrs/attrs/issues/756
        https://bugs.python.org/issue33188
        """
        from types import new_class
        from typing import Generic, TypeVar

        MyTypeVar = TypeVar("MyTypeVar")
        MyParent = new_class("MyParent", (Generic[MyTypeVar],), {})

        attr.make_class("test", {"id": attr.ib(type=str)}, (MyParent[int],))


class TestFields:
    """
    Tests for `fields`.
    """

    @given(simple_classes())
    def test_instance(self, C):
        """
        Raises `TypeError` on non-classes.
        """
        with pytest.raises(TypeError) as e:
            fields(C())

        assert "Passed object must be a class." == e.value.args[0]

    def test_handler_non_attrs_class(self):
        """
        Raises `ValueError` if passed a non-*attrs* instance.
        """
        with pytest.raises(NotAnAttrsClassError) as e:
            fields(object)

        assert (
            f"{object!r} is not an attrs-decorated class."
        ) == e.value.args[0]

    def test_handler_non_attrs_generic_class(self):
        """
        Raises `ValueError` if passed a non-*attrs* generic class.
        """
        T = TypeVar("T")

        class B(Generic[T]):
            pass

        with pytest.raises(NotAnAttrsClassError) as e:
            fields(B[str])

        assert (
            f"{B[str]!r} is not an attrs-decorated class."
        ) == e.value.args[0]

    @given(simple_classes())
    def test_fields(self, C):
        """
        Returns a list of `Attribute`a.
        """
        assert all(isinstance(a, Attribute) for a in fields(C))

    @given(simple_classes())
    def test_fields_properties(self, C):
        """
        Fields returns a tuple with properties.
        """
        for attribute in fields(C):
            assert getattr(fields(C), attribute.name) is attribute

    def test_generics(self):
        """
        Fields work with generic classes.
        """
        T = TypeVar("T")

        @attr.define
        class A(Generic[T]):
            a: T

        assert len(fields(A)) == 1
        assert fields(A).a.name == "a"
        assert fields(A).a.default is attr.NOTHING

        assert len(fields(A[str])) == 1
        assert fields(A[str]).a.name == "a"
        assert fields(A[str]).a.default is attr.NOTHING


class TestFieldsDict:
    """
    Tests for `fields_dict`.
    """

    @given(simple_classes())
    def test_instance(self, C):
        """
        Raises `TypeError` on non-classes.
        """
        with pytest.raises(TypeError) as e:
            fields_dict(C())

        assert "Passed object must be a class." == e.value.args[0]

    def test_handler_non_attrs_class(self):
        """
        Raises `ValueError` if passed a non-*attrs* instance.
        """
        with pytest.raises(NotAnAttrsClassError) as e:
            fields_dict(object)

        assert (
            f"{object!r} is not an attrs-decorated class."
        ) == e.value.args[0]

    @given(simple_classes())
    def test_fields_dict(self, C):
        """
        Returns an ordered dict of ``{attribute_name: Attribute}``.
        """
        d = fields_dict(C)

        assert isinstance(d, dict)
        assert list(fields(C)) == list(d.values())
        assert [a.name for a in fields(C)] == list(d)


class TestConverter:
    """
    Tests for attribute conversion.
    """

    def test_convert(self):
        """
        Return value of converter is used as the attribute's value.
        """
        C = make_class(
            "C", {"x": attr.ib(converter=lambda v: v + 1), "y": attr.ib()}
        )
        c = C(1, 2)

        assert c.x == 2
        assert c.y == 2

    @given(integers(), booleans())
    def test_convert_property(self, val, init):
        """
        Property tests for attributes using converter.
        """
        C = make_class(
            "C",
            {
                "y": attr.ib(),
                "x": attr.ib(
                    init=init, default=val, converter=lambda v: v + 1
                ),
            },
        )
        c = C(2)

        assert c.x == val + 1
        assert c.y == 2

    @given(integers(), booleans())
    def test_converter_factory_property(self, val, init):
        """
        Property tests for attributes with converter, and a factory default.
        """
        C = make_class(
            "C",
            {
                "y": attr.ib(),
                "x": attr.ib(
                    init=init,
                    default=Factory(lambda: val),
                    converter=lambda v: v + 1,
                ),
            },
        )
        c = C(2)

        assert c.x == val + 1
        assert c.y == 2

    def test_factory_takes_self(self):
        """
        If takes_self on factories is True, self is passed.
        """
        C = make_class(
            "C",
            {
                "x": attr.ib(
                    default=Factory((lambda self: self), takes_self=True)
                )
            },
        )

        i = C()

        assert i is i.x

    def test_factory_hashable(self):
        """
        Factory is hashable.
        """
        assert hash(Factory(None, False)) == hash(Factory(None, False))

    def test_convert_before_validate(self):
        """
        Validation happens after conversion.
        """

        def validator(inst, attr, val):
            raise RuntimeError("foo")

        C = make_class(
            "C",
            {
                "x": attr.ib(validator=validator, converter=lambda v: 1 / 0),
                "y": attr.ib(),
            },
        )
        with pytest.raises(ZeroDivisionError):
            C(1, 2)

    def test_frozen(self):
        """
        Converters circumvent immutability.
        """
        C = make_class(
            "C", {"x": attr.ib(converter=lambda v: int(v))}, frozen=True
        )
        C("1")


class TestValidate:
    """
    Tests for `validate`.
    """

    def test_success(self):
        """
        If the validator succeeds, nothing gets raised.
        """
        C = make_class(
            "C", {"x": attr.ib(validator=lambda *a: None), "y": attr.ib()}
        )
        validate(C(1, 2))

    def test_propagates(self):
        """
        The exception of the validator is handed through.
        """

        def raiser(_, __, value):
            if value == 42:
                raise FloatingPointError

        C = make_class("C", {"x": attr.ib(validator=raiser)})
        i = C(1)
        i.x = 42

        with pytest.raises(FloatingPointError):
            validate(i)

    def test_run_validators(self):
        """
        Setting `_run_validators` to False prevents validators from running.
        """
        _config._run_validators = False
        obj = object()

        def raiser(_, __, ___):
            raise Exception(obj)

        C = make_class("C", {"x": attr.ib(validator=raiser)})
        c = C(1)
        validate(c)
        assert 1 == c.x
        _config._run_validators = True

        with pytest.raises(Exception):
            validate(c)

        with pytest.raises(Exception) as e:
            C(1)
        assert (obj,) == e.value.args

    def test_multiple_validators(self):
        """
        If a list is passed as a validator, all of its items are treated as one
        and must pass.
        """

        def v1(_, __, value):
            if value == 23:
                raise TypeError("omg")

        def v2(_, __, value):
            if value == 42:
                raise ValueError("omg")

        C = make_class("C", {"x": attr.ib(validator=[v1, v2])})

        validate(C(1))

        with pytest.raises(TypeError) as e:
            C(23)

        assert "omg" == e.value.args[0]

        with pytest.raises(ValueError) as e:
            C(42)

        assert "omg" == e.value.args[0]

    def test_multiple_empty(self):
        """
        Empty list/tuple for validator is the same as None.
        """
        C1 = make_class("C", {"x": attr.ib(validator=[])})
        C2 = make_class("C", {"x": attr.ib(validator=None)})

        assert inspect.getsource(C1.__init__) == inspect.getsource(C2.__init__)


# Hypothesis seems to cache values, so the lists of attributes come out
# unsorted.
sorted_lists_of_attrs = list_of_attrs.map(
    lambda ln: sorted(ln, key=attrgetter("counter"))
)


class TestMetadata:
    """
    Tests for metadata handling.
    """

    @given(sorted_lists_of_attrs)
    def test_metadata_present(self, list_of_attrs):
        """
        Assert dictionaries are copied and present.
        """
        C = make_class("C", dict(zip(gen_attr_names(), list_of_attrs)))

        for hyp_attr, class_attr in zip(list_of_attrs, fields(C)):
            if hyp_attr.metadata is None:
                # The default is a singleton empty dict.
                assert class_attr.metadata is not None
                assert len(class_attr.metadata) == 0
            else:
                assert hyp_attr.metadata == class_attr.metadata

                # Once more, just to assert getting items and iteration.
                for k in class_attr.metadata:
                    assert hyp_attr.metadata[k] == class_attr.metadata[k]
                    assert hyp_attr.metadata.get(k) == class_attr.metadata.get(
                        k
                    )

    @given(simple_classes(), text())
    def test_metadata_immutability(self, C, string):
        """
        The metadata dict should be best-effort immutable.
        """
        for a in fields(C):
            with pytest.raises(TypeError):
                a.metadata[string] = string
            with pytest.raises(AttributeError):
                a.metadata.update({string: string})
            with pytest.raises(AttributeError):
                a.metadata.clear()
            with pytest.raises(AttributeError):
                a.metadata.setdefault(string, string)

            for k in a.metadata:
                # For some reason, MappingProxyType throws an IndexError for
                # deletes on a large integer key.
                with pytest.raises((TypeError, IndexError)):
                    del a.metadata[k]
                with pytest.raises(AttributeError):
                    a.metadata.pop(k)
            with pytest.raises(AttributeError):
                a.metadata.popitem()

    @given(lists(simple_attrs_without_metadata, min_size=2, max_size=5))
    def test_empty_metadata_singleton(self, list_of_attrs):
        """
        All empty metadata attributes share the same empty metadata dict.
        """
        C = make_class("C", dict(zip(gen_attr_names(), list_of_attrs)))
        for a in fields(C)[1:]:
            assert a.metadata is fields(C)[0].metadata

    @given(lists(simple_attrs_without_metadata, min_size=2, max_size=5))
    def test_empty_countingattr_metadata_independent(self, list_of_attrs):
        """
        All empty metadata attributes are independent before ``@attr.s``.
        """
        for x, y in itertools.combinations(list_of_attrs, 2):
            assert x.metadata is not y.metadata

    @given(lists(simple_attrs_with_metadata(), min_size=2, max_size=5))
    def test_not_none_metadata(self, list_of_attrs):
        """
        Non-empty metadata attributes exist as fields after ``@attr.s``.
        """
        C = make_class("C", dict(zip(gen_attr_names(), list_of_attrs)))

        assert len(fields(C)) > 0

        for cls_a, raw_a in zip(fields(C), list_of_attrs):
            assert cls_a.metadata != {}
            assert cls_a.metadata == raw_a.metadata

    def test_metadata(self):
        """
        If metadata that is not None is passed, it is used.

        This is necessary for coverage because the previous test is
        hypothesis-based.
        """
        md = {}
        a = attr.ib(metadata=md)

        assert md is a.metadata


class TestClassBuilder:
    """
    Tests for `_ClassBuilder`.
    """

    def test_repr_str(self):
        """
        Trying to add a `__str__` without having a `__repr__` raises a
        ValueError.
        """
        with pytest.raises(ValueError) as ei:
            make_class("C", {}, repr=False, str=True)

        assert (
            "__str__ can only be generated if a __repr__ exists.",
        ) == ei.value.args

    def test_repr(self):
        """
        repr of builder itself makes sense.
        """

        class C:
            pass

        b = _ClassBuilder(
            C,
            None,
            True,
            True,
            False,
            False,
            False,
            False,
            False,
            False,
            True,
            None,
            False,
            None,
        )

        assert "<_ClassBuilder(cls=C)>" == repr(b)

    def test_returns_self(self):
        """
        All methods return the builder for chaining.
        """

        class C:
            x = attr.ib()

        b = _ClassBuilder(
            C,
            None,
            True,
            True,
            False,
            False,
            False,
            False,
            False,
            False,
            True,
            None,
            False,
            None,
        )

        cls = (
            b.add_eq()
            .add_order()
            .add_hash()
            .add_init()
            .add_attrs_init()
            .add_repr("ns")
            .add_str()
            .build_class()
        )

        assert "ns.C(x=1)" == repr(cls(1))

    @pytest.mark.parametrize(
        "meth_name",
        [
            "__init__",
            "__hash__",
            "__repr__",
            "__str__",
            "__eq__",
            "__ne__",
            "__lt__",
            "__le__",
            "__gt__",
            "__ge__",
        ],
    )
    def test_attaches_meta_dunders(self, meth_name):
        """
        Generated methods have correct __module__, __name__, and __qualname__
        attributes.
        """

        @attr.s(hash=True, str=True)
        class C:
            def organic(self):
                pass

        @attr.s(hash=True, str=True)
        class D:
            pass

        meth_C = getattr(C, meth_name)
        meth_D = getattr(D, meth_name)

        assert meth_name == meth_C.__name__ == meth_D.__name__
        assert C.organic.__module__ == meth_C.__module__ == meth_D.__module__
        # This is assertion that would fail if a single __ne__ instance
        # was reused across multiple _make_eq calls.
        organic_prefix = C.organic.__qualname__.rsplit(".", 1)[0]
        assert organic_prefix + "." + meth_name == meth_C.__qualname__

    def test_handles_missing_meta_on_class(self):
        """
        If the class hasn't a __module__ or __qualname__, the method hasn't
        either.
        """

        class C:
            pass

        b = _ClassBuilder(
            C,
            these=None,
            slots=False,
            frozen=False,
            weakref_slot=True,
            getstate_setstate=False,
            auto_attribs=False,
            is_exc=False,
            kw_only=False,
            cache_hash=False,
            collect_by_mro=True,
            on_setattr=None,
            has_custom_setattr=False,
            field_transformer=None,
        )
        b._cls = {}  # no __module__; no __qualname__

        def fake_meth(self):
            pass

        fake_meth.__module__ = "42"
        fake_meth.__qualname__ = "23"

        rv = b._add_method_dunders(fake_meth)

        assert "42" == rv.__module__ == fake_meth.__module__
        assert "23" == rv.__qualname__ == fake_meth.__qualname__

    def test_weakref_setstate(self):
        """
        __weakref__ is not set on in setstate because it's not writable in
        slotted classes.
        """

        @attr.s(slots=True)
        class C:
            __weakref__ = attr.ib(
                init=False, hash=False, repr=False, eq=False, order=False
            )

        assert C() == copy.deepcopy(C())

    def test_no_references_to_original(self):
        """
        When subclassing a slotted class, there are no stray references to the
        original class.
        """

        @attr.s(slots=True)
        class C:
            pass

        @attr.s(slots=True)
        class C2(C):
            pass

        # The original C2 is in a reference cycle, so force a collect:
        gc.collect()

        assert [C2] == C.__subclasses__()

    def _get_copy_kwargs(include_slots=True):
        """
        Generate a list of compatible attr.s arguments for the `copy` tests.
        """
        options = ["frozen", "hash", "cache_hash"]

        if include_slots:
            options.extend(["slots", "weakref_slot"])

        out_kwargs = []
        for args in itertools.product([True, False], repeat=len(options)):
            kwargs = dict(zip(options, args))

            kwargs["hash"] = kwargs["hash"] or None

            if kwargs["cache_hash"] and not (
                kwargs["frozen"] or kwargs["hash"]
            ):
                continue

            out_kwargs.append(kwargs)

        return out_kwargs

    @pytest.mark.parametrize("kwargs", _get_copy_kwargs())
    def test_copy(self, kwargs):
        """
        Ensure that an attrs class can be copied successfully.
        """

        @attr.s(eq=True, **kwargs)
        class C:
            x = attr.ib()

        a = C(1)
        b = copy.deepcopy(a)

        assert a == b

    @pytest.mark.parametrize("kwargs", _get_copy_kwargs(include_slots=False))
    def test_copy_custom_setstate(self, kwargs):
        """
        Ensure that non-slots classes respect a custom __setstate__.
        """

        @attr.s(eq=True, **kwargs)
        class C:
            x = attr.ib()

            def __getstate__(self):
                return self.__dict__

            def __setstate__(self, state):
                state["x"] *= 5
                self.__dict__.update(state)

        expected = C(25)
        actual = copy.copy(C(5))

        assert actual == expected


class TestInitAlias:
    """
    Tests for Attribute alias handling.
    """

    def test_default_and_specify(self):
        """
        alias is present on the Attributes returned from attr.fields.

        If left unspecified, it defaults to standard private-attribute
        handling.  If specified, it passes through the explicit alias.
        """

        # alias is None by default on _CountingAttr
        default_counting = attr.ib()
        assert default_counting.alias is None

        override_counting = attr.ib(alias="specified")
        assert override_counting.alias == "specified"

        @attr.s
        class Cases:
            public_default = attr.ib()
            _private_default = attr.ib()
            __dunder_default__ = attr.ib()

            public_override = attr.ib(alias="public")
            _private_override = attr.ib(alias="_private")
            __dunder_override__ = attr.ib(alias="__dunder__")

        cases = attr.fields_dict(Cases)

        # Default applies private-name mangling logic
        assert cases["public_default"].name == "public_default"
        assert cases["public_default"].alias == "public_default"

        assert cases["_private_default"].name == "_private_default"
        assert cases["_private_default"].alias == "private_default"

        assert cases["__dunder_default__"].name == "__dunder_default__"
        assert cases["__dunder_default__"].alias == "dunder_default__"

        # Override is passed through
        assert cases["public_override"].name == "public_override"
        assert cases["public_override"].alias == "public"

        assert cases["_private_override"].name == "_private_override"
        assert cases["_private_override"].alias == "_private"

        assert cases["__dunder_override__"].name == "__dunder_override__"
        assert cases["__dunder_override__"].alias == "__dunder__"

        # And aliases are applied to the __init__ signature
        example = Cases(
            public_default=1,
            private_default=2,
            dunder_default__=3,
            public=4,
            _private=5,
            __dunder__=6,
        )

        assert example.public_default == 1
        assert example._private_default == 2
        assert example.__dunder_default__ == 3
        assert example.public_override == 4
        assert example._private_override == 5
        assert example.__dunder_override__ == 6

    def test_evolve(self):
        """
        attr.evolve uses Attribute.alias to determine parameter names.
        """

        @attr.s
        class EvolveCase:
            _override = attr.ib(alias="_override")
            __mangled = attr.ib()
            __dunder__ = attr.ib()

        org = EvolveCase(1, 2, 3)

        # Previous behavior of evolve as broken for double-underscore
        # passthrough, and would raise here due to mis-mapping the __dunder__
        # alias
        assert attr.evolve(org) == org

        # evolve uses the alias to match __init__ signature
        assert attr.evolve(
            org,
            _override=0,
        ) == EvolveCase(0, 2, 3)

        # and properly passes through dunders and mangles
        assert attr.evolve(
            org,
            EvolveCase__mangled=4,
            dunder__=5,
        ) == EvolveCase(1, 4, 5)


class TestMakeOrder:
    """
    Tests for _make_order().
    """

    def test_subclasses_cannot_be_compared(self):
        """
        Calling comparison methods on subclasses raises a TypeError.

        We use the actual operation so we get an error raised.
        """

        @attr.s
        class A:
            a = attr.ib()

        @attr.s
        class B(A):
            pass

        a = A(42)
        b = B(42)

        assert a <= a
        assert a >= a
        assert not a < a
        assert not a > a

        assert (
            NotImplemented
            == a.__lt__(b)
            == a.__le__(b)
            == a.__gt__(b)
            == a.__ge__(b)
        )

        with pytest.raises(TypeError):
            a <= b

        with pytest.raises(TypeError):
            a >= b

        with pytest.raises(TypeError):
            a < b

        with pytest.raises(TypeError):
            a > b


class TestDetermineAttrsEqOrder:
    def test_default(self):
        """
        If all are set to None, set both eq and order to the passed default.
        """
        assert (42, 42) == _determine_attrs_eq_order(None, None, None, 42)

    @pytest.mark.parametrize("eq", [True, False])
    def test_order_mirrors_eq_by_default(self, eq):
        """
        If order is None, it mirrors eq.
        """
        assert (eq, eq) == _determine_attrs_eq_order(None, eq, None, True)

    def test_order_without_eq(self):
        """
        eq=False, order=True raises a meaningful ValueError.
        """
        with pytest.raises(
            ValueError, match="`order` can only be True if `eq` is True too."
        ):
            _determine_attrs_eq_order(None, False, True, True)

    @given(cmp=booleans(), eq=optional_bool, order=optional_bool)
    def test_mix(self, cmp, eq, order):
        """
        If cmp is not None, eq and order must be None and vice versa.
        """
        assume(eq is not None or order is not None)

        with pytest.raises(
            ValueError, match="Don't mix `cmp` with `eq' and `order`."
        ):
            _determine_attrs_eq_order(cmp, eq, order, True)


class TestDetermineAttribEqOrder:
    def test_default(self):
        """
        If all are set to None, set both eq and order to the passed default.
        """
        assert (42, None, 42, None) == _determine_attrib_eq_order(
            None, None, None, 42
        )

    def test_eq_callable_order_boolean(self):
        """
        eq=callable or order=callable need to transformed into eq/eq_key
        or order/order_key.
        """
        assert (True, str.lower, False, None) == _determine_attrib_eq_order(
            None, str.lower, False, True
        )

    def test_eq_callable_order_callable(self):
        """
        eq=callable or order=callable need to transformed into eq/eq_key
        or order/order_key.
        """
        assert (True, str.lower, True, abs) == _determine_attrib_eq_order(
            None, str.lower, abs, True
        )

    def test_eq_boolean_order_callable(self):
        """
        eq=callable or order=callable need to transformed into eq/eq_key
        or order/order_key.
        """
        assert (True, None, True, str.lower) == _determine_attrib_eq_order(
            None, True, str.lower, True
        )

    @pytest.mark.parametrize("eq", [True, False])
    def test_order_mirrors_eq_by_default(self, eq):
        """
        If order is None, it mirrors eq.
        """
        assert (eq, None, eq, None) == _determine_attrib_eq_order(
            None, eq, None, True
        )

    def test_order_without_eq(self):
        """
        eq=False, order=True raises a meaningful ValueError.
        """
        with pytest.raises(
            ValueError, match="`order` can only be True if `eq` is True too."
        ):
            _determine_attrib_eq_order(None, False, True, True)

    @given(cmp=booleans(), eq=optional_bool, order=optional_bool)
    def test_mix(self, cmp, eq, order):
        """
        If cmp is not None, eq and order must be None and vice versa.
        """
        assume(eq is not None or order is not None)

        with pytest.raises(
            ValueError, match="Don't mix `cmp` with `eq' and `order`."
        ):
            _determine_attrib_eq_order(cmp, eq, order, True)


class TestDocs:
    @pytest.mark.parametrize(
        "meth_name",
        [
            "__init__",
            "__repr__",
            "__eq__",
            "__ne__",
            "__lt__",
            "__le__",
            "__gt__",
            "__ge__",
        ],
    )
    def test_docs(self, meth_name):
        """
        Tests the presence and correctness of the documentation
        for the generated methods
        """

        @attr.s
        class A:
            pass

        if hasattr(A, "__qualname__"):
            method = getattr(A, meth_name)
            expected = f"Method generated by attrs for class {A.__qualname__}."
            assert expected == method.__doc__


class BareC:
    pass


class BareSlottedC:
    __slots__ = ()


class TestAutoDetect:
    @pytest.mark.parametrize("C", [BareC, BareSlottedC])
    def test_determine_detects_non_presence_correctly(self, C):
        """
        On an empty class, nothing should be detected.
        """
        assert True is _determine_whether_to_implement(
            C, None, True, ("__init__",)
        )
        assert True is _determine_whether_to_implement(
            C, None, True, ("__repr__",)
        )
        assert True is _determine_whether_to_implement(
            C, None, True, ("__eq__", "__ne__")
        )
        assert True is _determine_whether_to_implement(
            C, None, True, ("__le__", "__lt__", "__ge__", "__gt__")
        )

    def test_make_all_by_default(self, slots, frozen):
        """
        If nothing is there to be detected, imply init=True, repr=True,
        hash=None, eq=True, order=True.
        """

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

        i = C(1)
        o = object()

        assert i.__init__ is not o.__init__
        assert i.__repr__ is not o.__repr__
        assert i.__eq__ is not o.__eq__
        assert i.__ne__ is not o.__ne__
        assert i.__le__ is not o.__le__
        assert i.__lt__ is not o.__lt__
        assert i.__ge__ is not o.__ge__
        assert i.__gt__ is not o.__gt__

    def test_detect_auto_init(self, slots, frozen):
        """
        If auto_detect=True and an __init__ exists, don't write one.
        """

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class CI:
            x = attr.ib()

            def __init__(self):
                object.__setattr__(self, "x", 42)

        assert 42 == CI().x

    def test_detect_auto_repr(self, slots, frozen):
        """
        If auto_detect=True and an __repr__ exists, don't write one.
        """

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __repr__(self):
                return "hi"

        assert "hi" == repr(C(42))

    def test_hash_uses_eq(self, slots, frozen):
        """
        If eq is passed in, then __hash__ should use the eq callable
        to generate the hash code.
        """

        @attr.s(slots=slots, frozen=frozen, hash=True)
        class C:
            x = attr.ib(eq=str)

        @attr.s(slots=slots, frozen=frozen, hash=True)
        class D:
            x = attr.ib()

        # These hashes should be the same because 1 is turned into
        # string before hashing.
        assert hash(C("1")) == hash(C(1))
        assert hash(D("1")) != hash(D(1))

    def test_detect_auto_hash(self, slots, frozen):
        """
        If auto_detect=True and an __hash__ exists, don't write one.
        """

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __hash__(self):
                return 0xC0FFEE

        assert 0xC0FFEE == hash(C(42))

    def test_detect_auto_eq(self, slots, frozen):
        """
        If auto_detect=True and an __eq__ or an __ne__, exist, don't write one.
        """

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __eq__(self, o):
                raise ValueError("worked")

        with pytest.raises(ValueError, match="worked"):
            C(1) == C(1)

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class D:
            x = attr.ib()

            def __ne__(self, o):
                raise ValueError("worked")

        with pytest.raises(ValueError, match="worked"):
            D(1) != D(1)

    def test_detect_auto_order(self, slots, frozen):
        """
        If auto_detect=True and an __ge__, __gt__, __le__, or and __lt__ exist,
        don't write one.

        It's surprisingly difficult to test this programmatically, so we do it
        by hand.
        """

        def assert_not_set(cls, ex, meth_name):
            __tracebackhide__ = True

            a = getattr(cls, meth_name)
            if meth_name == ex:
                assert a == 42
            else:
                assert a is getattr(object, meth_name)

        def assert_none_set(cls, ex):
            __tracebackhide__ = True

            for m in ("le", "lt", "ge", "gt"):
                assert_not_set(cls, ex, "__" + m + "__")

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class LE:
            __le__ = 42

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class LT:
            __lt__ = 42

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class GE:
            __ge__ = 42

        @attr.s(auto_detect=True, slots=slots, frozen=frozen)
        class GT:
            __gt__ = 42

        assert_none_set(LE, "__le__")
        assert_none_set(LT, "__lt__")
        assert_none_set(GE, "__ge__")
        assert_none_set(GT, "__gt__")

    def test_override_init(self, slots, frozen):
        """
        If init=True is passed, ignore __init__.
        """

        @attr.s(init=True, auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __init__(self):
                pytest.fail("should not be called")

        assert C(1) == C(1)

    def test_override_repr(self, slots, frozen):
        """
        If repr=True is passed, ignore __repr__.
        """

        @attr.s(repr=True, auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __repr__(self):
                pytest.fail("should not be called")

        assert "C(x=1)" == repr(C(1))

    def test_override_hash(self, slots, frozen):
        """
        If hash=True is passed, ignore __hash__.
        """

        @attr.s(hash=True, auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __hash__(self):
                pytest.fail("should not be called")

        assert hash(C(1))

    def test_override_eq(self, slots, frozen):
        """
        If eq=True is passed, ignore __eq__ and __ne__.
        """

        @attr.s(eq=True, auto_detect=True, slots=slots, frozen=frozen)
        class C:
            x = attr.ib()

            def __eq__(self, o):
                pytest.fail("should not be called")

            def __ne__(self, o):
                pytest.fail("should not be called")

        assert C(1) == C(1)

    @pytest.mark.parametrize(
        ("eq", "order", "cmp"),
        [
            (True, None, None),
            (True, True, None),
            (None, True, None),
            (None, None, True),
        ],
    )
    def test_override_order(self, slots, frozen, eq, order, cmp):
        """
        If order=True is passed, ignore __le__, __lt__, __gt__, __ge__.

        eq=True and cmp=True both imply order=True so test it too.
        """

        def meth(self, o):
            pytest.fail("should not be called")

        @attr.s(
            cmp=cmp,
            order=order,
            eq=eq,
            auto_detect=True,
            slots=slots,
            frozen=frozen,
        )
        class C:
            x = attr.ib()
            __le__ = __lt__ = __gt__ = __ge__ = meth

        assert C(1) < C(2)
        assert C(1) <= C(2)
        assert C(2) > C(1)
        assert C(2) >= C(1)

    @pytest.mark.parametrize("first", [True, False])
    def test_total_ordering(self, slots, first):
        """
        functools.total_ordering works as expected if an order method and an eq
        method are detected.

        Ensure the order doesn't matter.
        """

        class C:
            x = attr.ib()
            own_eq_called = attr.ib(default=False)
            own_le_called = attr.ib(default=False)

            def __eq__(self, o):
                self.own_eq_called = True
                return self.x == o.x

            def __le__(self, o):
                self.own_le_called = True
                return self.x <= o.x

        if first:
            C = functools.total_ordering(
                attr.s(auto_detect=True, slots=slots)(C)
            )
        else:
            C = attr.s(auto_detect=True, slots=slots)(
                functools.total_ordering(C)
            )

        c1, c2 = C(1), C(2)

        assert c1 < c2
        assert c1.own_le_called

        c1, c2 = C(1), C(2)

        assert c2 > c1
        assert c2.own_le_called

        c1, c2 = C(1), C(2)

        assert c2 != c1
        assert c1 == c1

        assert c1.own_eq_called

    def test_detects_setstate_getstate(self, slots):
        """
        __getstate__ and __setstate__ are not overwritten if either is present.
        """

        @attr.s(slots=slots, auto_detect=True)
        class C:
            def __getstate__(self):
                return ("hi",)

        assert getattr(object, "__setstate__", None) is getattr(
            C, "__setstate__", None
        )

        @attr.s(slots=slots, auto_detect=True)
        class C:
            called = attr.ib(False)

            def __setstate__(self, state):
                self.called = True

        i = C()

        assert False is i.called

        i.__setstate__(())

        assert True is i.called
        assert getattr(object, "__getstate__", None) is getattr(
            C, "__getstate__", None
        )

    @pytest.mark.skipif(PY310, reason="Pre-3.10 only.")
    def test_match_args_pre_310(self):
        """
        __match_args__ is not created on Python versions older than 3.10.
        """

        @attr.s
        class C:
            a = attr.ib()

        assert None is getattr(C, "__match_args__", None)


@pytest.mark.skipif(not PY310, reason="Structural pattern matching is 3.10+")
class TestMatchArgs:
    """
    Tests for match_args and __match_args__ generation.
    """

    def test_match_args(self):
        """
        __match_args__ is created by default on Python 3.10.
        """

        @attr.define
        class C:
            a = attr.field()

        assert ("a",) == C.__match_args__

    def test_explicit_match_args(self):
        """
        A custom __match_args__ set is not overwritten.
        """

        ma = ()

        @attr.define
        class C:
            a = attr.field()
            __match_args__ = ma

        assert C(42).__match_args__ is ma

    @pytest.mark.parametrize("match_args", [True, False])
    def test_match_args_attr_set(self, match_args):
        """
        __match_args__ is set depending on match_args.
        """

        @attr.define(match_args=match_args)
        class C:
            a = attr.field()

        if match_args:
            assert hasattr(C, "__match_args__")
        else:
            assert not hasattr(C, "__match_args__")

    def test_match_args_kw_only(self):
        """
        kw_only classes don't generate __match_args__.
        kw_only fields are not included in __match_args__.
        """

        @attr.define
        class C:
            a = attr.field(kw_only=True)
            b = attr.field()

        assert C.__match_args__ == ("b",)

        @attr.define(kw_only=True)
        class C:
            a = attr.field()
            b = attr.field()

        assert C.__match_args__ == ()

    def test_match_args_argument(self):
        """
        match_args being False with inheritance.
        """

        @attr.define(match_args=False)
        class X:
            a = attr.field()

        assert "__match_args__" not in X.__dict__

        @attr.define(match_args=False)
        class Y:
            a = attr.field()
            __match_args__ = ("b",)

        assert Y.__match_args__ == ("b",)

        @attr.define(match_args=False)
        class Z(Y):
            z = attr.field()

        assert Z.__match_args__ == ("b",)

        @attr.define
        class A:
            a = attr.field()
            z = attr.field()

        @attr.define(match_args=False)
        class B(A):
            b = attr.field()

        assert B.__match_args__ == ("a", "z")

    def test_make_class(self):
        """
        match_args generation with make_class.
        """

        C1 = make_class("C1", ["a", "b"])
        assert ("a", "b") == C1.__match_args__

        C1 = make_class("C1", ["a", "b"], match_args=False)
        assert not hasattr(C1, "__match_args__")

        C1 = make_class("C1", ["a", "b"], kw_only=True)
        assert () == C1.__match_args__

        C1 = make_class("C1", {"a": attr.ib(kw_only=True), "b": attr.ib()})
        assert ("b",) == C1.__match_args__
