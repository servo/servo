# SPDX-License-Identifier: MIT

"""
Tests for `attr.validators`.
"""


import re

import pytest

import attr

from attr import _config, fields, has
from attr import validators as validator_module
from attr.validators import (
    _subclass_of,
    and_,
    deep_iterable,
    deep_mapping,
    ge,
    gt,
    in_,
    instance_of,
    is_callable,
    le,
    lt,
    matches_re,
    max_len,
    min_len,
    not_,
    optional,
    provides,
)

from .utils import simple_attr


@pytest.fixture(scope="module")
def zope_interface():
    """Provides ``zope.interface`` if available, skipping the test if not."""
    try:
        import zope.interface
    except ImportError:
        raise pytest.skip(
            "zope-related tests skipped when zope.interface is not installed"
        )

    return zope.interface


class TestDisableValidators:
    @pytest.fixture(autouse=True)
    def _reset_default(self):
        """
        Make sure validators are always enabled after a test.
        """
        yield
        _config._run_validators = True

    def test_default(self):
        """
        Run validators by default.
        """
        assert _config._run_validators is True

    @pytest.mark.parametrize(
        ("value", "expected"), [(True, False), (False, True)]
    )
    def test_set_validators_disabled(self, value, expected):
        """
        Sets `_run_validators`.
        """
        validator_module.set_disabled(value)

        assert _config._run_validators is expected

    @pytest.mark.parametrize(
        ("value", "expected"), [(True, False), (False, True)]
    )
    def test_disabled(self, value, expected):
        """
        Returns `_run_validators`.
        """
        _config._run_validators = value

        assert validator_module.get_disabled() is expected

    def test_disabled_ctx(self):
        """
        The `disabled` context manager disables running validators,
        but only within its context.
        """
        assert _config._run_validators is True

        with validator_module.disabled():
            assert _config._run_validators is False

        assert _config._run_validators is True

    def test_disabled_ctx_with_errors(self):
        """
        Running validators is re-enabled even if an error is raised.
        """
        assert _config._run_validators is True

        with pytest.raises(ValueError), validator_module.disabled():
            assert _config._run_validators is False

            raise ValueError("haha!")

        assert _config._run_validators is True


class TestInstanceOf:
    """
    Tests for `instance_of`.
    """

    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert instance_of.__name__ in validator_module.__all__

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
            "'test' must be <class 'int'> (got '42' that is a <class 'str'>).",
            a,
            int,
            "42",
        ) == e.value.args

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = instance_of(int)
        assert ("<instance_of validator for type <class 'int'>>") == repr(v)


class TestMatchesRe:
    """
    Tests for `matches_re`.
    """

    def test_in_all(self):
        """
        validator is in ``__all__``.
        """
        assert matches_re.__name__ in validator_module.__all__

    def test_match(self):
        """
        Silent on matches, raises ValueError on mismatches.
        """

        @attr.s
        class ReTester:
            str_match = attr.ib(validator=matches_re("a|ab"))

        ReTester("ab")  # shouldn't raise exceptions
        with pytest.raises(TypeError):
            ReTester(1)
        with pytest.raises(ValueError):
            ReTester("1")
        with pytest.raises(ValueError):
            ReTester("a1")

    def test_flags(self):
        """
        Flags are propagated to the match function.
        """

        @attr.s
        class MatchTester:
            val = attr.ib(validator=matches_re("a", re.IGNORECASE, re.match))

        MatchTester("A1")  # test flags and using re.match

    def test_precompiled_pattern(self):
        """
        Pre-compiled patterns are accepted.
        """
        pattern = re.compile("a")

        @attr.s
        class RePatternTester:
            val = attr.ib(validator=matches_re(pattern))

        RePatternTester("a")

    def test_precompiled_pattern_no_flags(self):
        """
        A pre-compiled pattern cannot be combined with a 'flags' argument.
        """
        pattern = re.compile("")

        with pytest.raises(
            TypeError, match="can only be used with a string pattern"
        ):
            matches_re(pattern, flags=re.IGNORECASE)

    def test_different_func(self):
        """
        Changing the match functions works.
        """

        @attr.s
        class SearchTester:
            val = attr.ib(validator=matches_re("a", 0, re.search))

        SearchTester("bab")  # re.search will match

    def test_catches_invalid_func(self):
        """
        Invalid match functions are caught.
        """
        with pytest.raises(ValueError) as ei:
            matches_re("a", 0, lambda: None)

        assert (
            "'func' must be one of None, fullmatch, match, search."
            == ei.value.args[0]
        )

    @pytest.mark.parametrize(
        "func", [None, getattr(re, "fullmatch", None), re.match, re.search]
    )
    def test_accepts_all_valid_func(self, func):
        """
        Every valid match function is accepted.
        """
        matches_re("a", func=func)

    def test_repr(self):
        """
        __repr__ is meaningful.
        """
        assert repr(matches_re("a")).startswith(
            "<matches_re validator for pattern"
        )


def always_pass(_, __, ___):
    """
    Toy validator that always passes.
    """


def always_fail(_, __, ___):
    """
    Toy validator that always fails.
    """
    0 / 0


class TestAnd:
    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert and_.__name__ in validator_module.__all__

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
        class C:
            a1 = attr.ib("a1", validator=and_(instance_of(int)))
            a2 = attr.ib("a2", validator=[instance_of(int)])

        assert C.__attrs_attrs__[0].validator == C.__attrs_attrs__[1].validator


@pytest.fixture(scope="module")
def ifoo(zope_interface):
    """Provides a test ``zope.interface.Interface`` in ``zope`` tests."""

    class IFoo(zope_interface.Interface):
        """
        An interface.
        """

        def f():
            """
            A function called f.
            """

    return IFoo


class TestProvides:
    """
    Tests for `provides`.
    """

    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert provides.__name__ in validator_module.__all__

    def test_success(self, zope_interface, ifoo):
        """
        Nothing happens if value provides requested interface.
        """

        @zope_interface.implementer(ifoo)
        class C:
            def f(self):
                pass

        with pytest.deprecated_call():
            v = provides(ifoo)

        v(None, simple_attr("x"), C())

    def test_fail(self, ifoo):
        """
        Raises `TypeError` if interfaces isn't provided by value.
        """
        value = object()
        a = simple_attr("x")

        with pytest.deprecated_call():
            v = provides(ifoo)

        with pytest.raises(TypeError) as e:
            v(None, a, value)

        assert (
            f"'x' must provide {ifoo!r} which {value!r} doesn't.",
            a,
            ifoo,
            value,
        ) == e.value.args

    def test_repr(self, ifoo):
        """
        Returned validator has a useful `__repr__`.
        """
        with pytest.deprecated_call():
            v = provides(ifoo)

        assert (f"<provides validator for interface {ifoo!r}>") == repr(v)


@pytest.mark.parametrize(
    "validator",
    [
        instance_of(int),
        [always_pass, instance_of(int)],
        (always_pass, instance_of(int)),
    ],
)
class TestOptional:
    """
    Tests for `optional`.
    """

    def test_in_all(self, validator):
        """
        Verify that this validator is in ``__all__``.
        """
        assert optional.__name__ in validator_module.__all__

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
            "'test' must be <class 'int'> (got '42' that is a <class 'str'>).",
            a,
            int,
            "42",
        ) == e.value.args

    def test_repr(self, validator):
        """
        Returned validator has a useful `__repr__`.
        """
        v = optional(validator)

        if isinstance(validator, list):
            repr_s = (
                "<optional validator for _AndValidator(_validators=[{func}, "
                "<instance_of validator for type <class 'int'>>]) or None>"
            ).format(func=repr(always_pass))
        elif isinstance(validator, tuple):
            repr_s = (
                "<optional validator for _AndValidator(_validators=({func}, "
                "<instance_of validator for type <class 'int'>>)) or None>"
            ).format(func=repr(always_pass))
        else:
            repr_s = (
                "<optional validator for <instance_of validator for type "
                "<class 'int'>> or None>"
            )

        assert repr_s == repr(v)


class TestIn_:
    """
    Tests for `in_`.
    """

    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert in_.__name__ in validator_module.__all__

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
            a,
            [1, 2, 3],
            None,
        ) == e.value.args

    def test_fail_with_string(self):
        """
        Raise ValueError if the value is outside our options when the
        options are specified as a string and the value is not a string.
        """
        v = in_("abc")
        a = simple_attr("test")
        with pytest.raises(ValueError) as e:
            v(None, a, None)
        assert (
            "'test' must be in 'abc' (got None)",
            a,
            "abc",
            None,
        ) == e.value.args

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = in_([3, 4, 5])
        assert ("<in_ validator with options [3, 4, 5]>") == repr(v)


@pytest.fixture(
    name="member_validator",
    params=(
        instance_of(int),
        [always_pass, instance_of(int)],
        (always_pass, instance_of(int)),
    ),
    scope="module",
)
def _member_validator(request):
    """
    Provides sample `member_validator`s for some tests in `TestDeepIterable`
    """
    return request.param


class TestDeepIterable:
    """
    Tests for `deep_iterable`.
    """

    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert deep_iterable.__name__ in validator_module.__all__

    def test_success_member_only(self, member_validator):
        """
        If the member validator succeeds and the iterable validator is not set,
        nothing happens.
        """
        v = deep_iterable(member_validator)
        a = simple_attr("test")
        v(None, a, [42])

    def test_success_member_and_iterable(self, member_validator):
        """
        If both the member and iterable validators succeed, nothing happens.
        """
        iterable_validator = instance_of(list)
        v = deep_iterable(member_validator, iterable_validator)
        a = simple_attr("test")
        v(None, a, [42])

    @pytest.mark.parametrize(
        ("member_validator", "iterable_validator"),
        [
            (instance_of(int), 42),
            (42, instance_of(list)),
            (42, 42),
            (42, None),
            ([instance_of(int), 42], 42),
            ([42, instance_of(int)], 42),
        ],
    )
    def test_noncallable_validators(
        self, member_validator, iterable_validator
    ):
        """
        Raise `TypeError` if any validators are not callable.
        """
        with pytest.raises(TypeError) as e:
            deep_iterable(member_validator, iterable_validator)
        value = 42
        message = (
            f"must be callable (got {value} that is a {value.__class__})."
        )

        assert message in e.value.args[0]
        assert value == e.value.args[1]
        assert message in e.value.msg
        assert value == e.value.value

    def test_fail_invalid_member(self, member_validator):
        """
        Raise member validator error if an invalid member is found.
        """
        v = deep_iterable(member_validator)
        a = simple_attr("test")
        with pytest.raises(TypeError):
            v(None, a, [42, "42"])

    def test_fail_invalid_iterable(self, member_validator):
        """
        Raise iterable validator error if an invalid iterable is found.
        """
        member_validator = instance_of(int)
        iterable_validator = instance_of(tuple)
        v = deep_iterable(member_validator, iterable_validator)
        a = simple_attr("test")
        with pytest.raises(TypeError):
            v(None, a, [42])

    def test_fail_invalid_member_and_iterable(self, member_validator):
        """
        Raise iterable validator error if both the iterable
        and a member are invalid.
        """
        iterable_validator = instance_of(tuple)
        v = deep_iterable(member_validator, iterable_validator)
        a = simple_attr("test")
        with pytest.raises(TypeError):
            v(None, a, [42, "42"])

    def test_repr_member_only(self):
        """
        Returned validator has a useful `__repr__`
        when only member validator is set.
        """
        member_validator = instance_of(int)
        member_repr = "<instance_of validator for type <class 'int'>>"
        v = deep_iterable(member_validator)
        expected_repr = (
            f"<deep_iterable validator for iterables of {member_repr}>"
        )
        assert expected_repr == repr(v)

    def test_repr_member_only_sequence(self):
        """
        Returned validator has a useful `__repr__`
        when only member validator is set and the member validator is a list of
        validators
        """
        member_validator = [always_pass, instance_of(int)]
        member_repr = (
            f"_AndValidator(_validators=({always_pass!r}, "
            "<instance_of validator for type <class 'int'>>))"
        )
        v = deep_iterable(member_validator)
        expected_repr = (
            f"<deep_iterable validator for iterables of {member_repr}>"
        )
        assert expected_repr == repr(v)

    def test_repr_member_and_iterable(self):
        """
        Returned validator has a useful `__repr__` when both member
        and iterable validators are set.
        """
        member_validator = instance_of(int)
        member_repr = "<instance_of validator for type <class 'int'>>"
        iterable_validator = instance_of(list)
        iterable_repr = "<instance_of validator for type <class 'list'>>"
        v = deep_iterable(member_validator, iterable_validator)
        expected_repr = (
            "<deep_iterable validator for"
            f" {iterable_repr} iterables of {member_repr}>"
        )
        assert expected_repr == repr(v)

    def test_repr_sequence_member_and_iterable(self):
        """
        Returned validator has a useful `__repr__` when both member
        and iterable validators are set and the member validator is a list of
        validators
        """
        member_validator = [always_pass, instance_of(int)]
        member_repr = (
            f"_AndValidator(_validators=({always_pass!r}, "
            "<instance_of validator for type <class 'int'>>))"
        )
        iterable_validator = instance_of(list)
        iterable_repr = "<instance_of validator for type <class 'list'>>"
        v = deep_iterable(member_validator, iterable_validator)
        expected_repr = (
            "<deep_iterable validator for"
            f" {iterable_repr} iterables of {member_repr}>"
        )

        assert expected_repr == repr(v)


class TestDeepMapping:
    """
    Tests for `deep_mapping`.
    """

    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert deep_mapping.__name__ in validator_module.__all__

    def test_success(self):
        """
        If both the key and value validators succeed, nothing happens.
        """
        key_validator = instance_of(str)
        value_validator = instance_of(int)
        v = deep_mapping(key_validator, value_validator)
        a = simple_attr("test")
        v(None, a, {"a": 6, "b": 7})

    @pytest.mark.parametrize(
        ("key_validator", "value_validator", "mapping_validator"),
        [
            (42, instance_of(int), None),
            (instance_of(str), 42, None),
            (instance_of(str), instance_of(int), 42),
            (42, 42, None),
            (42, 42, 42),
        ],
    )
    def test_noncallable_validators(
        self, key_validator, value_validator, mapping_validator
    ):
        """
        Raise `TypeError` if any validators are not callable.
        """
        with pytest.raises(TypeError) as e:
            deep_mapping(key_validator, value_validator, mapping_validator)

        value = 42
        message = (
            f"must be callable (got {value} that is a {value.__class__})."
        )

        assert message in e.value.args[0]
        assert value == e.value.args[1]
        assert message in e.value.msg
        assert value == e.value.value

    def test_fail_invalid_mapping(self):
        """
        Raise `TypeError` if mapping validator fails.
        """
        key_validator = instance_of(str)
        value_validator = instance_of(int)
        mapping_validator = instance_of(dict)
        v = deep_mapping(key_validator, value_validator, mapping_validator)
        a = simple_attr("test")
        with pytest.raises(TypeError):
            v(None, a, None)

    def test_fail_invalid_key(self):
        """
        Raise key validator error if an invalid key is found.
        """
        key_validator = instance_of(str)
        value_validator = instance_of(int)
        v = deep_mapping(key_validator, value_validator)
        a = simple_attr("test")
        with pytest.raises(TypeError):
            v(None, a, {"a": 6, 42: 7})

    def test_fail_invalid_member(self):
        """
        Raise key validator error if an invalid member value is found.
        """
        key_validator = instance_of(str)
        value_validator = instance_of(int)
        v = deep_mapping(key_validator, value_validator)
        a = simple_attr("test")
        with pytest.raises(TypeError):
            v(None, a, {"a": "6", "b": 7})

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        key_validator = instance_of(str)
        key_repr = "<instance_of validator for type <class 'str'>>"
        value_validator = instance_of(int)
        value_repr = "<instance_of validator for type <class 'int'>>"
        v = deep_mapping(key_validator, value_validator)
        expected_repr = (
            "<deep_mapping validator for objects mapping "
            f"{key_repr} to {value_repr}>"
        )
        assert expected_repr == repr(v)


class TestIsCallable:
    """
    Tests for `is_callable`.
    """

    def test_in_all(self):
        """
        Verify that this validator is in ``__all__``.
        """
        assert is_callable.__name__ in validator_module.__all__

    def test_success(self):
        """
        If the value is callable, nothing happens.
        """
        v = is_callable()
        a = simple_attr("test")
        v(None, a, isinstance)

    def test_fail(self):
        """
        Raise TypeError if the value is not callable.
        """
        v = is_callable()
        a = simple_attr("test")
        with pytest.raises(TypeError) as e:
            v(None, a, None)

        value = None
        message = "'test' must be callable (got {value} that is a {type_})."
        expected_message = message.format(value=value, type_=value.__class__)

        assert expected_message == e.value.args[0]
        assert value == e.value.args[1]
        assert expected_message == e.value.msg
        assert value == e.value.value

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = is_callable()
        assert "<is_callable validator>" == repr(v)

    def test_exception_repr(self):
        """
        Verify that NotCallableError exception has a useful `__str__`.
        """
        from attr.exceptions import NotCallableError

        instance = NotCallableError(msg="Some Message", value=42)
        assert "Some Message" == str(instance)


def test_hashability():
    """
    Validator classes are hashable.
    """
    for obj_name in dir(validator_module):
        obj = getattr(validator_module, obj_name)
        if not has(obj):
            continue
        hash_func = getattr(obj, "__hash__", None)
        assert hash_func is not None
        assert hash_func is not object.__hash__


class TestLtLeGeGt:
    """
    Tests for `Lt, Le, Ge, Gt`.
    """

    BOUND = 4

    def test_in_all(self):
        """
        validator is in ``__all__``.
        """
        assert all(
            f.__name__ in validator_module.__all__ for f in [lt, le, ge, gt]
        )

    @pytest.mark.parametrize("v", [lt, le, ge, gt])
    def test_retrieve_bound(self, v):
        """
        The configured bound for the comparison can be extracted from the
        Attribute.
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=v(self.BOUND))

        assert fields(Tester).value.validator.bound == self.BOUND

    @pytest.mark.parametrize(
        ("v", "value"),
        [
            (lt, 3),
            (le, 3),
            (le, 4),
            (ge, 4),
            (ge, 5),
            (gt, 5),
        ],
    )
    def test_check_valid(self, v, value):
        """Silent if value {op} bound."""

        @attr.s
        class Tester:
            value = attr.ib(validator=v(self.BOUND))

        Tester(value)  # shouldn't raise exceptions

    @pytest.mark.parametrize(
        ("v", "value"),
        [
            (lt, 4),
            (le, 5),
            (ge, 3),
            (gt, 4),
        ],
    )
    def test_check_invalid(self, v, value):
        """Raise ValueError if value {op} bound."""

        @attr.s
        class Tester:
            value = attr.ib(validator=v(self.BOUND))

        with pytest.raises(ValueError):
            Tester(value)

    @pytest.mark.parametrize("v", [lt, le, ge, gt])
    def test_repr(self, v):
        """
        __repr__ is meaningful.
        """
        nv = v(23)
        assert repr(nv) == f"<Validator for x {nv.compare_op} {23}>"


class TestMaxLen:
    """
    Tests for `max_len`.
    """

    MAX_LENGTH = 4

    def test_in_all(self):
        """
        validator is in ``__all__``.
        """
        assert max_len.__name__ in validator_module.__all__

    def test_retrieve_max_len(self):
        """
        The configured max. length can be extracted from the Attribute
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=max_len(self.MAX_LENGTH))

        assert fields(Tester).value.validator.max_length == self.MAX_LENGTH

    @pytest.mark.parametrize(
        "value",
        [
            "",
            "foo",
            "spam",
            [],
            list(range(MAX_LENGTH)),
            {"spam": 3, "eggs": 4},
        ],
    )
    def test_check_valid(self, value):
        """
        Silent if len(value) <= max_len.
        Values can be strings and other iterables.
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=max_len(self.MAX_LENGTH))

        Tester(value)  # shouldn't raise exceptions

    @pytest.mark.parametrize(
        "value",
        [
            "bacon",
            list(range(6)),
        ],
    )
    def test_check_invalid(self, value):
        """
        Raise ValueError if len(value) > max_len.
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=max_len(self.MAX_LENGTH))

        with pytest.raises(ValueError):
            Tester(value)

    def test_repr(self):
        """
        __repr__ is meaningful.
        """
        assert repr(max_len(23)) == "<max_len validator for 23>"


class TestMinLen:
    """
    Tests for `min_len`.
    """

    MIN_LENGTH = 2

    def test_in_all(self):
        """
        validator is in ``__all__``.
        """
        assert min_len.__name__ in validator_module.__all__

    def test_retrieve_min_len(self):
        """
        The configured min. length can be extracted from the Attribute
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=min_len(self.MIN_LENGTH))

        assert fields(Tester).value.validator.min_length == self.MIN_LENGTH

    @pytest.mark.parametrize(
        "value",
        [
            "foo",
            "spam",
            list(range(MIN_LENGTH)),
            {"spam": 3, "eggs": 4},
        ],
    )
    def test_check_valid(self, value):
        """
        Silent if len(value) => min_len.
        Values can be strings and other iterables.
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=min_len(self.MIN_LENGTH))

        Tester(value)  # shouldn't raise exceptions

    @pytest.mark.parametrize(
        "value",
        [
            "",
            list(range(1)),
        ],
    )
    def test_check_invalid(self, value):
        """
        Raise ValueError if len(value) < min_len.
        """

        @attr.s
        class Tester:
            value = attr.ib(validator=min_len(self.MIN_LENGTH))

        with pytest.raises(ValueError):
            Tester(value)

    def test_repr(self):
        """
        __repr__ is meaningful.
        """
        assert repr(min_len(23)) == "<min_len validator for 23>"


class TestSubclassOf:
    """
    Tests for `_subclass_of`.
    """

    def test_success(self):
        """
        Nothing happens if classes match.
        """
        v = _subclass_of(int)
        v(None, simple_attr("test"), int)

    def test_subclass(self):
        """
        Subclasses are accepted too.
        """
        v = _subclass_of(int)
        # yep, bools are a subclass of int :(
        v(None, simple_attr("test"), bool)

    def test_fail(self):
        """
        Raises `TypeError` on wrong types.
        """
        v = _subclass_of(int)
        a = simple_attr("test")
        with pytest.raises(TypeError) as e:
            v(None, a, str)
        assert (
            "'test' must be a subclass of <class 'int'> (got <class 'str'>).",
            a,
            int,
            str,
        ) == e.value.args

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        v = _subclass_of(int)
        assert ("<subclass_of validator for type <class 'int'>>") == repr(v)


class TestNot_:
    """
    Tests for `not_`.
    """

    DEFAULT_EXC_TYPES = (ValueError, TypeError)

    def test_not_all(self):
        """
        The validator is in ``__all__``.
        """
        assert not_.__name__ in validator_module.__all__

    def test_repr(self):
        """
        Returned validator has a useful `__repr__`.
        """
        wrapped = in_([3, 4, 5])

        v = not_(wrapped)

        assert (
            f"<not_ validator wrapping {wrapped!r}, "
            f"capturing {v.exc_types!r}>"
        ) == repr(v)

    def test_success_because_fails(self):
        """
        If the wrapped validator fails, we're happy.
        """

        def always_fails(inst, attr, value):
            raise ValueError("always fails")

        v = not_(always_fails)
        a = simple_attr("test")
        input_value = 3

        v(1, a, input_value)

    def test_fails_because_success(self):
        """
        If the wrapped validator doesn't fail, not_ should fail.
        """

        def always_passes(inst, attr, value):
            pass

        v = not_(always_passes)
        a = simple_attr("test")
        input_value = 3

        with pytest.raises(ValueError) as e:
            v(1, a, input_value)

        assert (
            (
                "not_ validator child '{!r}' did not raise a captured error"
            ).format(always_passes),
            a,
            always_passes,
            input_value,
            self.DEFAULT_EXC_TYPES,
        ) == e.value.args

    def test_composable_with_in_pass(self):
        """
        Check something is ``not in`` something else.
        """
        v = not_(in_("abc"))
        a = simple_attr("test")
        input_value = "d"

        v(None, a, input_value)

    def test_composable_with_in_fail(self):
        """
        Check something is ``not in`` something else, but it is, so fail.
        """
        wrapped = in_("abc")
        v = not_(wrapped)
        a = simple_attr("test")
        input_value = "b"

        with pytest.raises(ValueError) as e:
            v(None, a, input_value)

        assert (
            (
                "not_ validator child '{!r}' did not raise a captured error"
            ).format(in_("abc")),
            a,
            wrapped,
            input_value,
            self.DEFAULT_EXC_TYPES,
        ) == e.value.args

    def test_composable_with_matches_re_pass(self):
        """
        Check something does not match a regex.
        """
        v = not_(matches_re("[a-z]{3}"))
        a = simple_attr("test")
        input_value = "spam"

        v(None, a, input_value)

    def test_composable_with_matches_re_fail(self):
        """
        Check something does not match a regex, but it does, so fail.
        """
        wrapped = matches_re("[a-z]{3}")
        v = not_(wrapped)
        a = simple_attr("test")
        input_value = "egg"

        with pytest.raises(ValueError) as e:
            v(None, a, input_value)

        assert (
            (
                f"not_ validator child '{wrapped!r}' did not raise a captured error"
            ),
            a,
            wrapped,
            input_value,
            self.DEFAULT_EXC_TYPES,
        ) == e.value.args

    def test_composable_with_instance_of_pass(self):
        """
        Check something is not a type. This validator raises a TypeError,
        rather than a ValueError like the others.
        """
        v = not_(instance_of((int, float)))
        a = simple_attr("test")

        v(None, a, "spam")

    def test_composable_with_instance_of_fail(self):
        """
        Check something is not a type, but it is, so fail.
        """
        wrapped = instance_of((int, float))
        v = not_(wrapped)
        a = simple_attr("test")
        input_value = 2.718281828

        with pytest.raises(ValueError) as e:
            v(None, a, input_value)

        assert (
            (
                "not_ validator child '{!r}' did not raise a captured error"
            ).format(instance_of((int, float))),
            a,
            wrapped,
            input_value,
            self.DEFAULT_EXC_TYPES,
        ) == e.value.args

    def test_custom_capture_match(self):
        """
        Match a custom exception provided to `not_`
        """
        v = not_(in_("abc"), exc_types=ValueError)
        a = simple_attr("test")

        v(None, a, "d")

    def test_custom_capture_miss(self):
        """
        If the exception doesn't match, the underlying raise comes through
        """

        class MyError(Exception):
            """:("""

        wrapped = in_("abc")
        v = not_(wrapped, exc_types=MyError)
        a = simple_attr("test")
        input_value = "d"

        with pytest.raises(ValueError) as e:
            v(None, a, input_value)

        # get the underlying exception to compare
        with pytest.raises(Exception) as e_from_wrapped:
            wrapped(None, a, input_value)
        assert e_from_wrapped.value.args == e.value.args

    def test_custom_msg(self):
        """
        If provided, use the custom message in the raised error
        """
        custom_msg = "custom message!"
        wrapped = in_("abc")
        v = not_(wrapped, msg=custom_msg)
        a = simple_attr("test")
        input_value = "a"

        with pytest.raises(ValueError) as e:
            v(None, a, input_value)

        assert (
            custom_msg,
            a,
            wrapped,
            input_value,
            self.DEFAULT_EXC_TYPES,
        ) == e.value.args

    def test_bad_exception_args(self):
        """
        Malformed exception arguments
        """
        wrapped = in_("abc")

        with pytest.raises(TypeError) as e:
            not_(wrapped, exc_types=(str, int))

        assert (
            "'exc_types' must be a subclass of <class 'Exception'> "
            "(got <class 'str'>)."
        ) == e.value.args[0]
