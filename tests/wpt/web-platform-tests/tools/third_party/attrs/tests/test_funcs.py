# SPDX-License-Identifier: MIT

"""
Tests for `attr._funcs`.
"""

from __future__ import absolute_import, division, print_function

from collections import OrderedDict

import pytest

from hypothesis import assume, given
from hypothesis import strategies as st

import attr

from attr import asdict, assoc, astuple, evolve, fields, has
from attr._compat import TYPE, Mapping, Sequence, ordered_dict
from attr.exceptions import AttrsAttributeNotFoundError
from attr.validators import instance_of

from .strategies import nested_classes, simple_classes


MAPPING_TYPES = (dict, OrderedDict)
SEQUENCE_TYPES = (list, tuple)


@pytest.fixture(scope="session", name="C")
def _C():
    """
    Return a simple but fully featured attrs class with an x and a y attribute.
    """
    import attr

    @attr.s
    class C(object):
        x = attr.ib()
        y = attr.ib()

    return C


class TestAsDict(object):
    """
    Tests for `asdict`.
    """

    @given(st.sampled_from(MAPPING_TYPES))
    def test_shallow(self, C, dict_factory):
        """
        Shallow asdict returns correct dict.
        """
        assert {"x": 1, "y": 2} == asdict(
            C(x=1, y=2), False, dict_factory=dict_factory
        )

    @given(st.sampled_from(MAPPING_TYPES))
    def test_recurse(self, C, dict_class):
        """
        Deep asdict returns correct dict.
        """
        assert {"x": {"x": 1, "y": 2}, "y": {"x": 3, "y": 4}} == asdict(
            C(C(1, 2), C(3, 4)), dict_factory=dict_class
        )

    def test_nested_lists(self, C):
        """
        Test unstructuring deeply nested lists.
        """
        inner = C(1, 2)
        outer = C([[inner]], None)

        assert {"x": [[{"x": 1, "y": 2}]], "y": None} == asdict(outer)

    def test_nested_dicts(self, C):
        """
        Test unstructuring deeply nested dictionaries.
        """
        inner = C(1, 2)
        outer = C({1: {2: inner}}, None)

        assert {"x": {1: {2: {"x": 1, "y": 2}}}, "y": None} == asdict(outer)

    @given(nested_classes, st.sampled_from(MAPPING_TYPES))
    def test_recurse_property(self, cls, dict_class):
        """
        Property tests for recursive asdict.
        """
        obj = cls()
        obj_dict = asdict(obj, dict_factory=dict_class)

        def assert_proper_dict_class(obj, obj_dict):
            assert isinstance(obj_dict, dict_class)

            for field in fields(obj.__class__):
                field_val = getattr(obj, field.name)
                if has(field_val.__class__):
                    # This field holds a class, recurse the assertions.
                    assert_proper_dict_class(field_val, obj_dict[field.name])
                elif isinstance(field_val, Sequence):
                    dict_val = obj_dict[field.name]
                    for item, item_dict in zip(field_val, dict_val):
                        if has(item.__class__):
                            assert_proper_dict_class(item, item_dict)
                elif isinstance(field_val, Mapping):
                    # This field holds a dictionary.
                    assert isinstance(obj_dict[field.name], dict_class)

                    for key, val in field_val.items():
                        if has(val.__class__):
                            assert_proper_dict_class(
                                val, obj_dict[field.name][key]
                            )

        assert_proper_dict_class(obj, obj_dict)

    @given(st.sampled_from(MAPPING_TYPES))
    def test_filter(self, C, dict_factory):
        """
        Attributes that are supposed to be skipped are skipped.
        """
        assert {"x": {"x": 1}} == asdict(
            C(C(1, 2), C(3, 4)),
            filter=lambda a, v: a.name != "y",
            dict_factory=dict_factory,
        )

    @given(container=st.sampled_from(SEQUENCE_TYPES))
    def test_lists_tuples(self, container, C):
        """
        If recurse is True, also recurse into lists.
        """
        assert {
            "x": 1,
            "y": [{"x": 2, "y": 3}, {"x": 4, "y": 5}, "a"],
        } == asdict(C(1, container([C(2, 3), C(4, 5), "a"])))

    @given(container=st.sampled_from(SEQUENCE_TYPES))
    def test_lists_tuples_retain_type(self, container, C):
        """
        If recurse and retain_collection_types are True, also recurse
        into lists and do not convert them into list.
        """
        assert {
            "x": 1,
            "y": container([{"x": 2, "y": 3}, {"x": 4, "y": 5}, "a"]),
        } == asdict(
            C(1, container([C(2, 3), C(4, 5), "a"])),
            retain_collection_types=True,
        )

    @given(set_type=st.sampled_from((set, frozenset)))
    def test_sets_no_retain(self, C, set_type):
        """
        Set types are converted to lists if retain_collection_types=False.
        """
        d = asdict(
            C(1, set_type((1, 2, 3))),
            retain_collection_types=False,
            recurse=True,
        )

        assert {"x": 1, "y": [1, 2, 3]} == d

    @given(st.sampled_from(MAPPING_TYPES))
    def test_dicts(self, C, dict_factory):
        """
        If recurse is True, also recurse into dicts.
        """
        res = asdict(C(1, {"a": C(4, 5)}), dict_factory=dict_factory)

        assert {"x": 1, "y": {"a": {"x": 4, "y": 5}}} == res
        assert isinstance(res, dict_factory)

    @given(simple_classes(private_attrs=False), st.sampled_from(MAPPING_TYPES))
    def test_roundtrip(self, cls, dict_class):
        """
        Test dumping to dicts and back for Hypothesis-generated classes.

        Private attributes don't round-trip (the attribute name is different
        than the initializer argument).
        """
        instance = cls()
        dict_instance = asdict(instance, dict_factory=dict_class)

        assert isinstance(dict_instance, dict_class)

        roundtrip_instance = cls(**dict_instance)

        assert instance == roundtrip_instance

    @given(simple_classes())
    def test_asdict_preserve_order(self, cls):
        """
        Field order should be preserved when dumping to an ordered_dict.
        """
        instance = cls()
        dict_instance = asdict(instance, dict_factory=ordered_dict)

        assert [a.name for a in fields(cls)] == list(dict_instance.keys())

    def test_retain_keys_are_tuples(self):
        """
        retain_collect_types also retains keys.
        """

        @attr.s
        class A(object):
            a = attr.ib()

        instance = A({(1,): 1})

        assert {"a": {(1,): 1}} == attr.asdict(
            instance, retain_collection_types=True
        )

    def test_tuple_keys(self):
        """
        If a key is collection type, retain_collection_types is False,
         the key is serialized as a tuple.

        See #646
        """

        @attr.s
        class A(object):
            a = attr.ib()

        instance = A({(1,): 1})

        assert {"a": {(1,): 1}} == attr.asdict(instance)


class TestAsTuple(object):
    """
    Tests for `astuple`.
    """

    @given(st.sampled_from(SEQUENCE_TYPES))
    def test_shallow(self, C, tuple_factory):
        """
        Shallow astuple returns correct dict.
        """
        assert tuple_factory([1, 2]) == astuple(
            C(x=1, y=2), False, tuple_factory=tuple_factory
        )

    @given(st.sampled_from(SEQUENCE_TYPES))
    def test_recurse(self, C, tuple_factory):
        """
        Deep astuple returns correct tuple.
        """
        assert tuple_factory(
            [tuple_factory([1, 2]), tuple_factory([3, 4])]
        ) == astuple(C(C(1, 2), C(3, 4)), tuple_factory=tuple_factory)

    @given(nested_classes, st.sampled_from(SEQUENCE_TYPES))
    def test_recurse_property(self, cls, tuple_class):
        """
        Property tests for recursive astuple.
        """
        obj = cls()
        obj_tuple = astuple(obj, tuple_factory=tuple_class)

        def assert_proper_tuple_class(obj, obj_tuple):
            assert isinstance(obj_tuple, tuple_class)
            for index, field in enumerate(fields(obj.__class__)):
                field_val = getattr(obj, field.name)
                if has(field_val.__class__):
                    # This field holds a class, recurse the assertions.
                    assert_proper_tuple_class(field_val, obj_tuple[index])

        assert_proper_tuple_class(obj, obj_tuple)

    @given(nested_classes, st.sampled_from(SEQUENCE_TYPES))
    def test_recurse_retain(self, cls, tuple_class):
        """
        Property tests for asserting collection types are retained.
        """
        obj = cls()
        obj_tuple = astuple(
            obj, tuple_factory=tuple_class, retain_collection_types=True
        )

        def assert_proper_col_class(obj, obj_tuple):
            # Iterate over all attributes, and if they are lists or mappings
            # in the original, assert they are the same class in the dumped.
            for index, field in enumerate(fields(obj.__class__)):
                field_val = getattr(obj, field.name)
                if has(field_val.__class__):
                    # This field holds a class, recurse the assertions.
                    assert_proper_col_class(field_val, obj_tuple[index])
                elif isinstance(field_val, (list, tuple)):
                    # This field holds a sequence of something.
                    expected_type = type(obj_tuple[index])
                    assert type(field_val) is expected_type
                    for obj_e, obj_tuple_e in zip(field_val, obj_tuple[index]):
                        if has(obj_e.__class__):
                            assert_proper_col_class(obj_e, obj_tuple_e)
                elif isinstance(field_val, dict):
                    orig = field_val
                    tupled = obj_tuple[index]
                    assert type(orig) is type(tupled)
                    for obj_e, obj_tuple_e in zip(
                        orig.items(), tupled.items()
                    ):
                        if has(obj_e[0].__class__):  # Dict key
                            assert_proper_col_class(obj_e[0], obj_tuple_e[0])
                        if has(obj_e[1].__class__):  # Dict value
                            assert_proper_col_class(obj_e[1], obj_tuple_e[1])

        assert_proper_col_class(obj, obj_tuple)

    @given(st.sampled_from(SEQUENCE_TYPES))
    def test_filter(self, C, tuple_factory):
        """
        Attributes that are supposed to be skipped are skipped.
        """
        assert tuple_factory([tuple_factory([1])]) == astuple(
            C(C(1, 2), C(3, 4)),
            filter=lambda a, v: a.name != "y",
            tuple_factory=tuple_factory,
        )

    @given(container=st.sampled_from(SEQUENCE_TYPES))
    def test_lists_tuples(self, container, C):
        """
        If recurse is True, also recurse into lists.
        """
        assert (1, [(2, 3), (4, 5), "a"]) == astuple(
            C(1, container([C(2, 3), C(4, 5), "a"]))
        )

    @given(st.sampled_from(SEQUENCE_TYPES))
    def test_dicts(self, C, tuple_factory):
        """
        If recurse is True, also recurse into dicts.
        """
        res = astuple(C(1, {"a": C(4, 5)}), tuple_factory=tuple_factory)
        assert tuple_factory([1, {"a": tuple_factory([4, 5])}]) == res
        assert isinstance(res, tuple_factory)

    @given(container=st.sampled_from(SEQUENCE_TYPES))
    def test_lists_tuples_retain_type(self, container, C):
        """
        If recurse and retain_collection_types are True, also recurse
        into lists and do not convert them into list.
        """
        assert (1, container([(2, 3), (4, 5), "a"])) == astuple(
            C(1, container([C(2, 3), C(4, 5), "a"])),
            retain_collection_types=True,
        )

    @given(container=st.sampled_from(MAPPING_TYPES))
    def test_dicts_retain_type(self, container, C):
        """
        If recurse and retain_collection_types are True, also recurse
        into lists and do not convert them into list.
        """
        assert (1, container({"a": (4, 5)})) == astuple(
            C(1, container({"a": C(4, 5)})), retain_collection_types=True
        )

    @given(simple_classes(), st.sampled_from(SEQUENCE_TYPES))
    def test_roundtrip(self, cls, tuple_class):
        """
        Test dumping to tuple and back for Hypothesis-generated classes.
        """
        instance = cls()
        tuple_instance = astuple(instance, tuple_factory=tuple_class)

        assert isinstance(tuple_instance, tuple_class)

        roundtrip_instance = cls(*tuple_instance)

        assert instance == roundtrip_instance

    @given(set_type=st.sampled_from((set, frozenset)))
    def test_sets_no_retain(self, C, set_type):
        """
        Set types are converted to lists if retain_collection_types=False.
        """
        d = astuple(
            C(1, set_type((1, 2, 3))),
            retain_collection_types=False,
            recurse=True,
        )

        assert (1, [1, 2, 3]) == d


class TestHas(object):
    """
    Tests for `has`.
    """

    def test_positive(self, C):
        """
        Returns `True` on decorated classes.
        """
        assert has(C)

    def test_positive_empty(self):
        """
        Returns `True` on decorated classes even if there are no attributes.
        """

        @attr.s
        class D(object):
            pass

        assert has(D)

    def test_negative(self):
        """
        Returns `False` on non-decorated classes.
        """
        assert not has(object)


class TestAssoc(object):
    """
    Tests for `assoc`.
    """

    @given(slots=st.booleans(), frozen=st.booleans())
    def test_empty(self, slots, frozen):
        """
        Empty classes without changes get copied.
        """

        @attr.s(slots=slots, frozen=frozen)
        class C(object):
            pass

        i1 = C()
        with pytest.deprecated_call():
            i2 = assoc(i1)

        assert i1 is not i2
        assert i1 == i2

    @given(simple_classes())
    def test_no_changes(self, C):
        """
        No changes means a verbatim copy.
        """
        i1 = C()
        with pytest.deprecated_call():
            i2 = assoc(i1)

        assert i1 is not i2
        assert i1 == i2

    @given(simple_classes(), st.data())
    def test_change(self, C, data):
        """
        Changes work.
        """
        # Take the first attribute, and change it.
        assume(fields(C))  # Skip classes with no attributes.
        field_names = [a.name for a in fields(C)]
        original = C()
        chosen_names = data.draw(st.sets(st.sampled_from(field_names)))
        change_dict = {name: data.draw(st.integers()) for name in chosen_names}

        with pytest.deprecated_call():
            changed = assoc(original, **change_dict)

        for k, v in change_dict.items():
            assert getattr(changed, k) == v

    @given(simple_classes())
    def test_unknown(self, C):
        """
        Wanting to change an unknown attribute raises an
        AttrsAttributeNotFoundError.
        """
        # No generated class will have a four letter attribute.
        with pytest.raises(
            AttrsAttributeNotFoundError
        ) as e, pytest.deprecated_call():
            assoc(C(), aaaa=2)

        assert (
            "aaaa is not an attrs attribute on {cls!r}.".format(cls=C),
        ) == e.value.args

    def test_frozen(self):
        """
        Works on frozen classes.
        """

        @attr.s(frozen=True)
        class C(object):
            x = attr.ib()
            y = attr.ib()

        with pytest.deprecated_call():
            assert C(3, 2) == assoc(C(1, 2), x=3)

    def test_warning(self):
        """
        DeprecationWarning points to the correct file.
        """

        @attr.s
        class C(object):
            x = attr.ib()

        with pytest.warns(DeprecationWarning) as wi:
            assert C(2) == assoc(C(1), x=2)

        assert __file__ == wi.list[0].filename


class TestEvolve(object):
    """
    Tests for `evolve`.
    """

    @given(slots=st.booleans(), frozen=st.booleans())
    def test_empty(self, slots, frozen):
        """
        Empty classes without changes get copied.
        """

        @attr.s(slots=slots, frozen=frozen)
        class C(object):
            pass

        i1 = C()
        i2 = evolve(i1)

        assert i1 is not i2
        assert i1 == i2

    @given(simple_classes())
    def test_no_changes(self, C):
        """
        No changes means a verbatim copy.
        """
        i1 = C()
        i2 = evolve(i1)

        assert i1 is not i2
        assert i1 == i2

    @given(simple_classes(), st.data())
    def test_change(self, C, data):
        """
        Changes work.
        """
        # Take the first attribute, and change it.
        assume(fields(C))  # Skip classes with no attributes.
        field_names = [a.name for a in fields(C)]
        original = C()
        chosen_names = data.draw(st.sets(st.sampled_from(field_names)))
        # We pay special attention to private attributes, they should behave
        # like in `__init__`.
        change_dict = {
            name.replace("_", ""): data.draw(st.integers())
            for name in chosen_names
        }
        changed = evolve(original, **change_dict)
        for name in chosen_names:
            assert getattr(changed, name) == change_dict[name.replace("_", "")]

    @given(simple_classes())
    def test_unknown(self, C):
        """
        Wanting to change an unknown attribute raises an
        AttrsAttributeNotFoundError.
        """
        # No generated class will have a four letter attribute.
        with pytest.raises(TypeError) as e:
            evolve(C(), aaaa=2)

        if hasattr(C, "__attrs_init__"):
            expected = (
                "__attrs_init__() got an unexpected keyword argument 'aaaa'"
            )
        else:
            expected = "__init__() got an unexpected keyword argument 'aaaa'"

        assert e.value.args[0].endswith(expected)

    def test_validator_failure(self):
        """
        TypeError isn't swallowed when validation fails within evolve.
        """

        @attr.s
        class C(object):
            a = attr.ib(validator=instance_of(int))

        with pytest.raises(TypeError) as e:
            evolve(C(a=1), a="some string")
        m = e.value.args[0]

        assert m.startswith("'a' must be <{type} 'int'>".format(type=TYPE))

    def test_private(self):
        """
        evolve() acts as `__init__` with regards to private attributes.
        """

        @attr.s
        class C(object):
            _a = attr.ib()

        assert evolve(C(1), a=2)._a == 2

        with pytest.raises(TypeError):
            evolve(C(1), _a=2)

        with pytest.raises(TypeError):
            evolve(C(1), a=3, _a=2)

    def test_non_init_attrs(self):
        """
        evolve() handles `init=False` attributes.
        """

        @attr.s
        class C(object):
            a = attr.ib()
            b = attr.ib(init=False, default=0)

        assert evolve(C(1), a=2).a == 2

    def test_regression_attrs_classes(self):
        """
        evolve() can evolve fields that are instances of attrs classes.

        Regression test for #804
        """

        @attr.s
        class Cls1(object):
            param1 = attr.ib()

        @attr.s
        class Cls2(object):
            param2 = attr.ib()

        obj2a = Cls2(param2="a")
        obj2b = Cls2(param2="b")

        obj1a = Cls1(param1=obj2a)

        assert Cls1(param1=Cls2(param2="b")) == attr.evolve(
            obj1a, param1=obj2b
        )

    def test_dicts(self):
        """
        evolve() can replace an attrs class instance with a dict.

        See #806
        """

        @attr.s
        class Cls1(object):
            param1 = attr.ib()

        @attr.s
        class Cls2(object):
            param2 = attr.ib()

        obj2a = Cls2(param2="a")
        obj2b = {"foo": 42, "param2": 42}

        obj1a = Cls1(param1=obj2a)

        assert Cls1({"foo": 42, "param2": 42}) == attr.evolve(
            obj1a, param1=obj2b
        )
