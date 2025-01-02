# SPDX-License-Identifier: MIT

"""
Tests for methods from `attrib._cmp`.
"""


import pytest

from attr._cmp import cmp_using


# Test parameters.
EqCSameType = cmp_using(eq=lambda a, b: a == b, class_name="EqCSameType")
PartialOrderCSameType = cmp_using(
    eq=lambda a, b: a == b,
    lt=lambda a, b: a < b,
    class_name="PartialOrderCSameType",
)
FullOrderCSameType = cmp_using(
    eq=lambda a, b: a == b,
    lt=lambda a, b: a < b,
    le=lambda a, b: a <= b,
    gt=lambda a, b: a > b,
    ge=lambda a, b: a >= b,
    class_name="FullOrderCSameType",
)

EqCAnyType = cmp_using(
    eq=lambda a, b: a == b, require_same_type=False, class_name="EqCAnyType"
)
PartialOrderCAnyType = cmp_using(
    eq=lambda a, b: a == b,
    lt=lambda a, b: a < b,
    require_same_type=False,
    class_name="PartialOrderCAnyType",
)


eq_data = [
    (EqCSameType, True),
    (EqCAnyType, False),
]

order_data = [
    (PartialOrderCSameType, True),
    (PartialOrderCAnyType, False),
    (FullOrderCSameType, True),
]

eq_ids = [c[0].__name__ for c in eq_data]
order_ids = [c[0].__name__ for c in order_data]

cmp_data = eq_data + order_data
cmp_ids = eq_ids + order_ids


class TestEqOrder:
    """
    Tests for eq and order related methods.
    """

    #########
    # eq
    #########
    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), cmp_data, ids=cmp_ids
    )
    def test_equal_same_type(self, cls, requires_same_type):
        """
        Equal objects are detected as equal.
        """
        assert cls(1) == cls(1)
        assert not (cls(1) != cls(1))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), cmp_data, ids=cmp_ids
    )
    def test_unequal_same_type(self, cls, requires_same_type):
        """
        Unequal objects of correct type are detected as unequal.
        """
        assert cls(1) != cls(2)
        assert not (cls(1) == cls(2))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), cmp_data, ids=cmp_ids
    )
    def test_equal_different_type(self, cls, requires_same_type):
        """
        Equal values of different types are detected appropriately.
        """
        assert (cls(1) == cls(1.0)) == (not requires_same_type)
        assert not (cls(1) != cls(1.0)) == (not requires_same_type)

    #########
    # lt
    #########
    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), eq_data, ids=eq_ids
    )
    def test_lt_unorderable(self, cls, requires_same_type):
        """
        TypeError is raised if class does not implement __lt__.
        """
        with pytest.raises(TypeError):
            cls(1) < cls(2)

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_lt_same_type(self, cls, requires_same_type):
        """
        Less-than objects are detected appropriately.
        """
        assert cls(1) < cls(2)
        assert not (cls(2) < cls(1))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_not_lt_same_type(self, cls, requires_same_type):
        """
        Not less-than objects are detected appropriately.
        """
        assert cls(2) >= cls(1)
        assert not (cls(1) >= cls(2))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_lt_different_type(self, cls, requires_same_type):
        """
        Less-than values of different types are detected appropriately.
        """
        if requires_same_type:
            # Unlike __eq__, NotImplemented will cause an exception to be
            # raised from __lt__.
            with pytest.raises(TypeError):
                cls(1) < cls(2.0)
        else:
            assert cls(1) < cls(2.0)
            assert not (cls(2) < cls(1.0))

    #########
    # le
    #########
    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), eq_data, ids=eq_ids
    )
    def test_le_unorderable(self, cls, requires_same_type):
        """
        TypeError is raised if class does not implement __le__.
        """
        with pytest.raises(TypeError):
            cls(1) <= cls(2)

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_le_same_type(self, cls, requires_same_type):
        """
        Less-than-or-equal objects are detected appropriately.
        """
        assert cls(1) <= cls(1)
        assert cls(1) <= cls(2)
        assert not (cls(2) <= cls(1))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_not_le_same_type(self, cls, requires_same_type):
        """
        Not less-than-or-equal objects are detected appropriately.
        """
        assert cls(2) > cls(1)
        assert not (cls(1) > cls(1))
        assert not (cls(1) > cls(2))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_le_different_type(self, cls, requires_same_type):
        """
        Less-than-or-equal values of diff. types are detected appropriately.
        """
        if requires_same_type:
            # Unlike __eq__, NotImplemented will cause an exception to be
            # raised from __le__.
            with pytest.raises(TypeError):
                cls(1) <= cls(2.0)
        else:
            assert cls(1) <= cls(2.0)
            assert cls(1) <= cls(1.0)
            assert not (cls(2) <= cls(1.0))

    #########
    # gt
    #########
    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), eq_data, ids=eq_ids
    )
    def test_gt_unorderable(self, cls, requires_same_type):
        """
        TypeError is raised if class does not implement __gt__.
        """
        with pytest.raises(TypeError):
            cls(2) > cls(1)

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_gt_same_type(self, cls, requires_same_type):
        """
        Greater-than objects are detected appropriately.
        """
        assert cls(2) > cls(1)
        assert not (cls(1) > cls(2))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_not_gt_same_type(self, cls, requires_same_type):
        """
        Not greater-than objects are detected appropriately.
        """
        assert cls(1) <= cls(2)
        assert not (cls(2) <= cls(1))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_gt_different_type(self, cls, requires_same_type):
        """
        Greater-than values of different types are detected appropriately.
        """
        if requires_same_type:
            # Unlike __eq__, NotImplemented will cause an exception to be
            # raised from __gt__.
            with pytest.raises(TypeError):
                cls(2) > cls(1.0)
        else:
            assert cls(2) > cls(1.0)
            assert not (cls(1) > cls(2.0))

    #########
    # ge
    #########
    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), eq_data, ids=eq_ids
    )
    def test_ge_unorderable(self, cls, requires_same_type):
        """
        TypeError is raised if class does not implement __ge__.
        """
        with pytest.raises(TypeError):
            cls(2) >= cls(1)

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_ge_same_type(self, cls, requires_same_type):
        """
        Greater-than-or-equal objects are detected appropriately.
        """
        assert cls(1) >= cls(1)
        assert cls(2) >= cls(1)
        assert not (cls(1) >= cls(2))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_not_ge_same_type(self, cls, requires_same_type):
        """
        Not greater-than-or-equal objects are detected appropriately.
        """
        assert cls(1) < cls(2)
        assert not (cls(1) < cls(1))
        assert not (cls(2) < cls(1))

    @pytest.mark.parametrize(
        ("cls", "requires_same_type"), order_data, ids=order_ids
    )
    def test_ge_different_type(self, cls, requires_same_type):
        """
        Greater-than-or-equal values of diff. types are detected appropriately.
        """
        if requires_same_type:
            # Unlike __eq__, NotImplemented will cause an exception to be
            # raised from __ge__.
            with pytest.raises(TypeError):
                cls(2) >= cls(1.0)
        else:
            assert cls(2) >= cls(2.0)
            assert cls(2) >= cls(1.0)
            assert not (cls(1) >= cls(2.0))


class TestDundersUnnamedClass:
    """
    Tests for dunder attributes of unnamed classes.
    """

    cls = cmp_using(eq=lambda a, b: a == b)

    def test_class(self):
        """
        Class name and qualified name should be well behaved.
        """
        assert self.cls.__name__ == "Comparable"
        assert self.cls.__qualname__ == "Comparable"

    def test_eq(self):
        """
        __eq__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__eq__
        assert method.__doc__.strip() == "Return a == b.  Computed by attrs."
        assert method.__name__ == "__eq__"

    def test_ne(self):
        """
        __ne__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__ne__
        assert method.__doc__.strip() == (
            "Check equality and either forward a NotImplemented or\n"
            "        return the result negated."
        )
        assert method.__name__ == "__ne__"


class TestTotalOrderingException:
    """
    Test for exceptions related to total ordering.
    """

    def test_eq_must_specified(self):
        """
        `total_ordering` requires `__eq__` to be specified.
        """
        with pytest.raises(ValueError) as ei:
            cmp_using(lt=lambda a, b: a < b)

        assert ei.value.args[0] == (
            "eq must be define is order to complete ordering from "
            "lt, le, gt, ge."
        )


class TestNotImplementedIsPropagated:
    """
    Test related to functions that return NotImplemented.
    """

    def test_not_implemented_is_propagated(self):
        """
        If the comparison function returns NotImplemented,
        the dunder method should too.
        """
        C = cmp_using(eq=lambda a, b: NotImplemented if a == 1 else a == b)

        assert C(2) == C(2)
        assert C(1) != C(1)


class TestDundersPartialOrdering:
    """
    Tests for dunder attributes of classes with partial ordering.
    """

    cls = PartialOrderCSameType

    def test_class(self):
        """
        Class name and qualified name should be well behaved.
        """
        assert self.cls.__name__ == "PartialOrderCSameType"
        assert self.cls.__qualname__ == "PartialOrderCSameType"

    def test_eq(self):
        """
        __eq__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__eq__
        assert method.__doc__.strip() == "Return a == b.  Computed by attrs."
        assert method.__name__ == "__eq__"

    def test_ne(self):
        """
        __ne__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__ne__
        assert method.__doc__.strip() == (
            "Check equality and either forward a NotImplemented or\n"
            "        return the result negated."
        )
        assert method.__name__ == "__ne__"

    def test_lt(self):
        """
        __lt__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__lt__
        assert method.__doc__.strip() == "Return a < b.  Computed by attrs."
        assert method.__name__ == "__lt__"

    def test_le(self):
        """
        __le__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__le__
        assert method.__doc__.strip().startswith(
            "Return a <= b.  Computed by @total_ordering from"
        )
        assert method.__name__ == "__le__"

    def test_gt(self):
        """
        __gt__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__gt__
        assert method.__doc__.strip().startswith(
            "Return a > b.  Computed by @total_ordering from"
        )
        assert method.__name__ == "__gt__"

    def test_ge(self):
        """
        __ge__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__ge__
        assert method.__doc__.strip().startswith(
            "Return a >= b.  Computed by @total_ordering from"
        )
        assert method.__name__ == "__ge__"


class TestDundersFullOrdering:
    """
    Tests for dunder attributes of classes with full ordering.
    """

    cls = FullOrderCSameType

    def test_class(self):
        """
        Class name and qualified name should be well behaved.
        """
        assert self.cls.__name__ == "FullOrderCSameType"
        assert self.cls.__qualname__ == "FullOrderCSameType"

    def test_eq(self):
        """
        __eq__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__eq__
        assert method.__doc__.strip() == "Return a == b.  Computed by attrs."
        assert method.__name__ == "__eq__"

    def test_ne(self):
        """
        __ne__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__ne__
        assert method.__doc__.strip() == (
            "Check equality and either forward a NotImplemented or\n"
            "        return the result negated."
        )
        assert method.__name__ == "__ne__"

    def test_lt(self):
        """
        __lt__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__lt__
        assert method.__doc__.strip() == "Return a < b.  Computed by attrs."
        assert method.__name__ == "__lt__"

    def test_le(self):
        """
        __le__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__le__
        assert method.__doc__.strip() == "Return a <= b.  Computed by attrs."
        assert method.__name__ == "__le__"

    def test_gt(self):
        """
        __gt__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__gt__
        assert method.__doc__.strip() == "Return a > b.  Computed by attrs."
        assert method.__name__ == "__gt__"

    def test_ge(self):
        """
        __ge__ docstring and qualified name should be well behaved.
        """
        method = self.cls.__ge__
        assert method.__doc__.strip() == "Return a >= b.  Computed by attrs."
        assert method.__name__ == "__ge__"
