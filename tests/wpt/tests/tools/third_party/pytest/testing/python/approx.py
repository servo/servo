import operator
import sys
from contextlib import contextmanager
from decimal import Decimal
from fractions import Fraction
from operator import eq
from operator import ne
from typing import Optional

import pytest
from _pytest.pytester import Pytester
from pytest import approx

inf, nan = float("inf"), float("nan")


@pytest.fixture
def mocked_doctest_runner(monkeypatch):
    import doctest

    class MockedPdb:
        def __init__(self, out):
            pass

        def set_trace(self):
            raise NotImplementedError("not used")

        def reset(self):
            pass

        def set_continue(self):
            pass

    monkeypatch.setattr("doctest._OutputRedirectingPdb", MockedPdb)

    class MyDocTestRunner(doctest.DocTestRunner):
        def report_failure(self, out, test, example, got):
            raise AssertionError(
                "'{}' evaluates to '{}', not '{}'".format(
                    example.source.strip(), got.strip(), example.want.strip()
                )
            )

    return MyDocTestRunner()


@contextmanager
def temporary_verbosity(config, verbosity=0):
    original_verbosity = config.getoption("verbose")
    config.option.verbose = verbosity
    try:
        yield
    finally:
        config.option.verbose = original_verbosity


@pytest.fixture
def assert_approx_raises_regex(pytestconfig):
    def do_assert(lhs, rhs, expected_message, verbosity_level=0):
        import re

        with temporary_verbosity(pytestconfig, verbosity_level):
            with pytest.raises(AssertionError) as e:
                assert lhs == approx(rhs)

        nl = "\n"
        obtained_message = str(e.value).splitlines()[1:]
        assert len(obtained_message) == len(expected_message), (
            "Regex message length doesn't match obtained.\n"
            "Obtained:\n"
            f"{nl.join(obtained_message)}\n\n"
            "Expected regex:\n"
            f"{nl.join(expected_message)}\n\n"
        )

        for i, (obtained_line, expected_line) in enumerate(
            zip(obtained_message, expected_message)
        ):
            regex = re.compile(expected_line)
            assert regex.match(obtained_line) is not None, (
                "Unexpected error message:\n"
                f"{nl.join(obtained_message)}\n\n"
                "Did not match regex:\n"
                f"{nl.join(expected_message)}\n\n"
                f"With verbosity level = {verbosity_level}, on line {i}"
            )

    return do_assert


SOME_FLOAT = r"[+-]?([0-9]*[.])?[0-9]+\s*"
SOME_INT = r"[0-9]+\s*"


class TestApprox:
    def test_error_messages(self, assert_approx_raises_regex):
        np = pytest.importorskip("numpy")

        assert_approx_raises_regex(
            2.0,
            1.0,
            [
                "  comparison failed",
                f"  Obtained: {SOME_FLOAT}",
                f"  Expected: {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

        assert_approx_raises_regex(
            {"a": 1.0, "b": 1000.0, "c": 1000000.0},
            {
                "a": 2.0,
                "b": 1000.0,
                "c": 3000000.0,
            },
            [
                r"  comparison failed. Mismatched elements: 2 / 3:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                rf"  Max relative difference: {SOME_FLOAT}",
                r"  Index \| Obtained\s+\| Expected           ",
                rf"  a     \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
                rf"  c     \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

        assert_approx_raises_regex(
            [1.0, 2.0, 3.0, 4.0],
            [1.0, 3.0, 3.0, 5.0],
            [
                r"  comparison failed. Mismatched elements: 2 / 4:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                rf"  Max relative difference: {SOME_FLOAT}",
                r"  Index \| Obtained\s+\| Expected   ",
                rf"  1     \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
                rf"  3     \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

        a = np.linspace(0, 100, 20)
        b = np.linspace(0, 100, 20)
        a[10] += 0.5
        assert_approx_raises_regex(
            a,
            b,
            [
                r"  comparison failed. Mismatched elements: 1 / 20:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                rf"  Max relative difference: {SOME_FLOAT}",
                r"  Index \| Obtained\s+\| Expected",
                rf"  \(10,\) \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

        assert_approx_raises_regex(
            np.array(
                [
                    [[1.1987311, 12412342.3], [3.214143244, 1423412423415.677]],
                    [[1, 2], [3, 219371297321973]],
                ]
            ),
            np.array(
                [
                    [[1.12313, 12412342.3], [3.214143244, 534523542345.677]],
                    [[1, 2], [3, 7]],
                ]
            ),
            [
                r"  comparison failed. Mismatched elements: 3 / 8:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                rf"  Max relative difference: {SOME_FLOAT}",
                r"  Index\s+\| Obtained\s+\| Expected\s+",
                rf"  \(0, 0, 0\) \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
                rf"  \(0, 1, 1\) \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
                rf"  \(1, 1, 1\) \| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

        # Specific test for comparison with 0.0 (relative diff will be 'inf')
        assert_approx_raises_regex(
            [0.0],
            [1.0],
            [
                r"  comparison failed. Mismatched elements: 1 / 1:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                r"  Max relative difference: inf",
                r"  Index \| Obtained\s+\| Expected   ",
                rf"\s*0\s*\| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

        assert_approx_raises_regex(
            np.array([0.0]),
            np.array([1.0]),
            [
                r"  comparison failed. Mismatched elements: 1 / 1:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                r"  Max relative difference: inf",
                r"  Index \| Obtained\s+\| Expected   ",
                rf"\s*\(0,\)\s*\| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
            ],
        )

    def test_error_messages_invalid_args(self, assert_approx_raises_regex):
        np = pytest.importorskip("numpy")
        with pytest.raises(AssertionError) as e:
            assert np.array([[1.2, 3.4], [4.0, 5.0]]) == pytest.approx(
                np.array([[4.0], [5.0]])
            )
        message = "\n".join(str(e.value).split("\n")[1:])
        assert message == "\n".join(
            [
                "  Impossible to compare arrays with different shapes.",
                "  Shapes: (2, 1) and (2, 2)",
            ]
        )

        with pytest.raises(AssertionError) as e:
            assert [1.0, 2.0, 3.0] == pytest.approx([4.0, 5.0])
        message = "\n".join(str(e.value).split("\n")[1:])
        assert message == "\n".join(
            [
                "  Impossible to compare lists with different sizes.",
                "  Lengths: 2 and 3",
            ]
        )

    def test_error_messages_with_different_verbosity(self, assert_approx_raises_regex):
        np = pytest.importorskip("numpy")
        for v in [0, 1, 2]:
            # Verbosity level doesn't affect the error message for scalars
            assert_approx_raises_regex(
                2.0,
                1.0,
                [
                    "  comparison failed",
                    f"  Obtained: {SOME_FLOAT}",
                    f"  Expected: {SOME_FLOAT} ± {SOME_FLOAT}",
                ],
                verbosity_level=v,
            )

        a = np.linspace(1, 101, 20)
        b = np.linspace(2, 102, 20)
        assert_approx_raises_regex(
            a,
            b,
            [
                r"  comparison failed. Mismatched elements: 20 / 20:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                rf"  Max relative difference: {SOME_FLOAT}",
                r"  Index \| Obtained\s+\| Expected",
                rf"  \(0,\)\s+\| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
                rf"  \(1,\)\s+\| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}",
                rf"  \(2,\)\s+\| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}...",
                "",
                rf"\s*...Full output truncated \({SOME_INT} lines hidden\), use '-vv' to show",
            ],
            verbosity_level=0,
        )

        assert_approx_raises_regex(
            a,
            b,
            [
                r"  comparison failed. Mismatched elements: 20 / 20:",
                rf"  Max absolute difference: {SOME_FLOAT}",
                rf"  Max relative difference: {SOME_FLOAT}",
                r"  Index \| Obtained\s+\| Expected",
            ]
            + [
                rf"  \({i},\)\s+\| {SOME_FLOAT} \| {SOME_FLOAT} ± {SOME_FLOAT}"
                for i in range(20)
            ],
            verbosity_level=2,
        )

    def test_repr_string(self):
        assert repr(approx(1.0)) == "1.0 ± 1.0e-06"
        assert repr(approx([1.0, 2.0])) == "approx([1.0 ± 1.0e-06, 2.0 ± 2.0e-06])"
        assert repr(approx((1.0, 2.0))) == "approx((1.0 ± 1.0e-06, 2.0 ± 2.0e-06))"
        assert repr(approx(inf)) == "inf"
        assert repr(approx(1.0, rel=nan)) == "1.0 ± ???"
        assert repr(approx(1.0, rel=inf)) == "1.0 ± inf"

        # Dictionaries aren't ordered, so we need to check both orders.
        assert repr(approx({"a": 1.0, "b": 2.0})) in (
            "approx({'a': 1.0 ± 1.0e-06, 'b': 2.0 ± 2.0e-06})",
            "approx({'b': 2.0 ± 2.0e-06, 'a': 1.0 ± 1.0e-06})",
        )

    def test_repr_complex_numbers(self):
        assert repr(approx(inf + 1j)) == "(inf+1j)"
        assert repr(approx(1.0j, rel=inf)) == "1j ± inf"

        # can't compute a sensible tolerance
        assert repr(approx(nan + 1j)) == "(nan+1j) ± ???"

        assert repr(approx(1.0j)) == "1j ± 1.0e-06 ∠ ±180°"

        # relative tolerance is scaled to |3+4j| = 5
        assert repr(approx(3 + 4 * 1j)) == "(3+4j) ± 5.0e-06 ∠ ±180°"

        # absolute tolerance is not scaled
        assert repr(approx(3.3 + 4.4 * 1j, abs=0.02)) == "(3.3+4.4j) ± 2.0e-02 ∠ ±180°"

    @pytest.mark.parametrize(
        "value, expected_repr_string",
        [
            (5.0, "approx(5.0 ± 5.0e-06)"),
            ([5.0], "approx([5.0 ± 5.0e-06])"),
            ([[5.0]], "approx([[5.0 ± 5.0e-06]])"),
            ([[5.0, 6.0]], "approx([[5.0 ± 5.0e-06, 6.0 ± 6.0e-06]])"),
            ([[5.0], [6.0]], "approx([[5.0 ± 5.0e-06], [6.0 ± 6.0e-06]])"),
        ],
    )
    def test_repr_nd_array(self, value, expected_repr_string):
        """Make sure that arrays of all different dimensions are repr'd correctly."""
        np = pytest.importorskip("numpy")
        np_array = np.array(value)
        assert repr(approx(np_array)) == expected_repr_string

    def test_bool(self):
        with pytest.raises(AssertionError) as err:
            assert approx(1)

        assert err.match(r"approx\(\) is not supported in a boolean context")

    def test_operator_overloading(self):
        assert 1 == approx(1, rel=1e-6, abs=1e-12)
        assert not (1 != approx(1, rel=1e-6, abs=1e-12))
        assert 10 != approx(1, rel=1e-6, abs=1e-12)
        assert not (10 == approx(1, rel=1e-6, abs=1e-12))

    def test_exactly_equal(self):
        examples = [
            (2.0, 2.0),
            (0.1e200, 0.1e200),
            (1.123e-300, 1.123e-300),
            (12345, 12345.0),
            (0.0, -0.0),
            (345678, 345678),
            (Decimal("1.0001"), Decimal("1.0001")),
            (Fraction(1, 3), Fraction(-1, -3)),
        ]
        for a, x in examples:
            assert a == approx(x)

    def test_opposite_sign(self):
        examples = [(eq, 1e-100, -1e-100), (ne, 1e100, -1e100)]
        for op, a, x in examples:
            assert op(a, approx(x))

    def test_zero_tolerance(self):
        within_1e10 = [(1.1e-100, 1e-100), (-1.1e-100, -1e-100)]
        for a, x in within_1e10:
            assert x == approx(x, rel=0.0, abs=0.0)
            assert a != approx(x, rel=0.0, abs=0.0)
            assert a == approx(x, rel=0.0, abs=5e-101)
            assert a != approx(x, rel=0.0, abs=5e-102)
            assert a == approx(x, rel=5e-1, abs=0.0)
            assert a != approx(x, rel=5e-2, abs=0.0)

    @pytest.mark.parametrize(
        ("rel", "abs"),
        [
            (-1e100, None),
            (None, -1e100),
            (1e100, -1e100),
            (-1e100, 1e100),
            (-1e100, -1e100),
        ],
    )
    def test_negative_tolerance(
        self, rel: Optional[float], abs: Optional[float]
    ) -> None:
        # Negative tolerances are not allowed.
        with pytest.raises(ValueError):
            1.1 == approx(1, rel, abs)

    def test_negative_tolerance_message(self):
        # Error message for negative tolerance should include the value.
        with pytest.raises(ValueError, match="-3"):
            0 == approx(1, abs=-3)
        with pytest.raises(ValueError, match="-3"):
            0 == approx(1, rel=-3)

    def test_inf_tolerance(self):
        # Everything should be equal if the tolerance is infinite.
        large_diffs = [(1, 1000), (1e-50, 1e50), (-1.0, -1e300), (0.0, 10)]
        for a, x in large_diffs:
            assert a != approx(x, rel=0.0, abs=0.0)
            assert a == approx(x, rel=inf, abs=0.0)
            assert a == approx(x, rel=0.0, abs=inf)
            assert a == approx(x, rel=inf, abs=inf)

    def test_inf_tolerance_expecting_zero(self) -> None:
        # If the relative tolerance is zero but the expected value is infinite,
        # the actual tolerance is a NaN, which should be an error.
        with pytest.raises(ValueError):
            1 == approx(0, rel=inf, abs=0.0)
        with pytest.raises(ValueError):
            1 == approx(0, rel=inf, abs=inf)

    def test_nan_tolerance(self) -> None:
        with pytest.raises(ValueError):
            1.1 == approx(1, rel=nan)
        with pytest.raises(ValueError):
            1.1 == approx(1, abs=nan)
        with pytest.raises(ValueError):
            1.1 == approx(1, rel=nan, abs=nan)

    def test_reasonable_defaults(self):
        # Whatever the defaults are, they should work for numbers close to 1
        # than have a small amount of floating-point error.
        assert 0.1 + 0.2 == approx(0.3)

    def test_default_tolerances(self):
        # This tests the defaults as they are currently set.  If you change the
        # defaults, this test will fail but you should feel free to change it.
        # None of the other tests (except the doctests) should be affected by
        # the choice of defaults.
        examples = [
            # Relative tolerance used.
            (eq, 1e100 + 1e94, 1e100),
            (ne, 1e100 + 2e94, 1e100),
            (eq, 1e0 + 1e-6, 1e0),
            (ne, 1e0 + 2e-6, 1e0),
            # Absolute tolerance used.
            (eq, 1e-100, +1e-106),
            (eq, 1e-100, +2e-106),
            (eq, 1e-100, 0),
        ]
        for op, a, x in examples:
            assert op(a, approx(x))

    def test_custom_tolerances(self):
        assert 1e8 + 1e0 == approx(1e8, rel=5e-8, abs=5e0)
        assert 1e8 + 1e0 == approx(1e8, rel=5e-9, abs=5e0)
        assert 1e8 + 1e0 == approx(1e8, rel=5e-8, abs=5e-1)
        assert 1e8 + 1e0 != approx(1e8, rel=5e-9, abs=5e-1)

        assert 1e0 + 1e-8 == approx(1e0, rel=5e-8, abs=5e-8)
        assert 1e0 + 1e-8 == approx(1e0, rel=5e-9, abs=5e-8)
        assert 1e0 + 1e-8 == approx(1e0, rel=5e-8, abs=5e-9)
        assert 1e0 + 1e-8 != approx(1e0, rel=5e-9, abs=5e-9)

        assert 1e-8 + 1e-16 == approx(1e-8, rel=5e-8, abs=5e-16)
        assert 1e-8 + 1e-16 == approx(1e-8, rel=5e-9, abs=5e-16)
        assert 1e-8 + 1e-16 == approx(1e-8, rel=5e-8, abs=5e-17)
        assert 1e-8 + 1e-16 != approx(1e-8, rel=5e-9, abs=5e-17)

    def test_relative_tolerance(self):
        within_1e8_rel = [(1e8 + 1e0, 1e8), (1e0 + 1e-8, 1e0), (1e-8 + 1e-16, 1e-8)]
        for a, x in within_1e8_rel:
            assert a == approx(x, rel=5e-8, abs=0.0)
            assert a != approx(x, rel=5e-9, abs=0.0)

    def test_absolute_tolerance(self):
        within_1e8_abs = [(1e8 + 9e-9, 1e8), (1e0 + 9e-9, 1e0), (1e-8 + 9e-9, 1e-8)]
        for a, x in within_1e8_abs:
            assert a == approx(x, rel=0, abs=5e-8)
            assert a != approx(x, rel=0, abs=5e-9)

    def test_expecting_zero(self):
        examples = [
            (ne, 1e-6, 0.0),
            (ne, -1e-6, 0.0),
            (eq, 1e-12, 0.0),
            (eq, -1e-12, 0.0),
            (ne, 2e-12, 0.0),
            (ne, -2e-12, 0.0),
            (ne, inf, 0.0),
            (ne, nan, 0.0),
        ]
        for op, a, x in examples:
            assert op(a, approx(x, rel=0.0, abs=1e-12))
            assert op(a, approx(x, rel=1e-6, abs=1e-12))

    def test_expecting_inf(self):
        examples = [
            (eq, inf, inf),
            (eq, -inf, -inf),
            (ne, inf, -inf),
            (ne, 0.0, inf),
            (ne, nan, inf),
        ]
        for op, a, x in examples:
            assert op(a, approx(x))

    def test_expecting_nan(self):
        examples = [
            (eq, nan, nan),
            (eq, -nan, -nan),
            (eq, nan, -nan),
            (ne, 0.0, nan),
            (ne, inf, nan),
        ]
        for op, a, x in examples:
            # Nothing is equal to NaN by default.
            assert a != approx(x)

            # If ``nan_ok=True``, then NaN is equal to NaN.
            assert op(a, approx(x, nan_ok=True))

    def test_int(self):
        within_1e6 = [(1000001, 1000000), (-1000001, -1000000)]
        for a, x in within_1e6:
            assert a == approx(x, rel=5e-6, abs=0)
            assert a != approx(x, rel=5e-7, abs=0)
            assert approx(x, rel=5e-6, abs=0) == a
            assert approx(x, rel=5e-7, abs=0) != a

    def test_decimal(self):
        within_1e6 = [
            (Decimal("1.000001"), Decimal("1.0")),
            (Decimal("-1.000001"), Decimal("-1.0")),
        ]
        for a, x in within_1e6:
            assert a == approx(x)
            assert a == approx(x, rel=Decimal("5e-6"), abs=0)
            assert a != approx(x, rel=Decimal("5e-7"), abs=0)
            assert approx(x, rel=Decimal("5e-6"), abs=0) == a
            assert approx(x, rel=Decimal("5e-7"), abs=0) != a

    def test_fraction(self):
        within_1e6 = [
            (1 + Fraction(1, 1000000), Fraction(1)),
            (-1 - Fraction(-1, 1000000), Fraction(-1)),
        ]
        for a, x in within_1e6:
            assert a == approx(x, rel=5e-6, abs=0)
            assert a != approx(x, rel=5e-7, abs=0)
            assert approx(x, rel=5e-6, abs=0) == a
            assert approx(x, rel=5e-7, abs=0) != a

    def test_complex(self):
        within_1e6 = [
            (1.000001 + 1.0j, 1.0 + 1.0j),
            (1.0 + 1.000001j, 1.0 + 1.0j),
            (-1.000001 + 1.0j, -1.0 + 1.0j),
            (1.0 - 1.000001j, 1.0 - 1.0j),
        ]
        for a, x in within_1e6:
            assert a == approx(x, rel=5e-6, abs=0)
            assert a != approx(x, rel=5e-7, abs=0)
            assert approx(x, rel=5e-6, abs=0) == a
            assert approx(x, rel=5e-7, abs=0) != a

    def test_list(self):
        actual = [1 + 1e-7, 2 + 1e-8]
        expected = [1, 2]

        # Return false if any element is outside the tolerance.
        assert actual == approx(expected, rel=5e-7, abs=0)
        assert actual != approx(expected, rel=5e-8, abs=0)
        assert approx(expected, rel=5e-7, abs=0) == actual
        assert approx(expected, rel=5e-8, abs=0) != actual

    def test_list_decimal(self):
        actual = [Decimal("1.000001"), Decimal("2.000001")]
        expected = [Decimal("1"), Decimal("2")]

        assert actual == approx(expected)

    def test_list_wrong_len(self):
        assert [1, 2] != approx([1])
        assert [1, 2] != approx([1, 2, 3])

    def test_tuple(self):
        actual = (1 + 1e-7, 2 + 1e-8)
        expected = (1, 2)

        # Return false if any element is outside the tolerance.
        assert actual == approx(expected, rel=5e-7, abs=0)
        assert actual != approx(expected, rel=5e-8, abs=0)
        assert approx(expected, rel=5e-7, abs=0) == actual
        assert approx(expected, rel=5e-8, abs=0) != actual

    def test_tuple_wrong_len(self):
        assert (1, 2) != approx((1,))
        assert (1, 2) != approx((1, 2, 3))

    def test_tuple_vs_other(self):
        assert 1 != approx((1,))

    def test_dict(self):
        actual = {"a": 1 + 1e-7, "b": 2 + 1e-8}
        # Dictionaries became ordered in python3.6, so switch up the order here
        # to make sure it doesn't matter.
        expected = {"b": 2, "a": 1}

        # Return false if any element is outside the tolerance.
        assert actual == approx(expected, rel=5e-7, abs=0)
        assert actual != approx(expected, rel=5e-8, abs=0)
        assert approx(expected, rel=5e-7, abs=0) == actual
        assert approx(expected, rel=5e-8, abs=0) != actual

    def test_dict_decimal(self):
        actual = {"a": Decimal("1.000001"), "b": Decimal("2.000001")}
        # Dictionaries became ordered in python3.6, so switch up the order here
        # to make sure it doesn't matter.
        expected = {"b": Decimal("2"), "a": Decimal("1")}

        assert actual == approx(expected)

    def test_dict_wrong_len(self):
        assert {"a": 1, "b": 2} != approx({"a": 1})
        assert {"a": 1, "b": 2} != approx({"a": 1, "c": 2})
        assert {"a": 1, "b": 2} != approx({"a": 1, "b": 2, "c": 3})

    def test_dict_nonnumeric(self):
        assert {"a": 1.0, "b": None} == pytest.approx({"a": 1.0, "b": None})
        assert {"a": 1.0, "b": 1} != pytest.approx({"a": 1.0, "b": None})

    def test_dict_vs_other(self):
        assert 1 != approx({"a": 0})

    def test_numpy_array(self):
        np = pytest.importorskip("numpy")

        actual = np.array([1 + 1e-7, 2 + 1e-8])
        expected = np.array([1, 2])

        # Return false if any element is outside the tolerance.
        assert actual == approx(expected, rel=5e-7, abs=0)
        assert actual != approx(expected, rel=5e-8, abs=0)
        assert approx(expected, rel=5e-7, abs=0) == expected
        assert approx(expected, rel=5e-8, abs=0) != actual

        # Should be able to compare lists with numpy arrays.
        assert list(actual) == approx(expected, rel=5e-7, abs=0)
        assert list(actual) != approx(expected, rel=5e-8, abs=0)
        assert actual == approx(list(expected), rel=5e-7, abs=0)
        assert actual != approx(list(expected), rel=5e-8, abs=0)

    def test_numpy_tolerance_args(self):
        """
        Check that numpy rel/abs args are handled correctly
        for comparison against an np.array
        Check both sides of the operator, hopefully it doesn't impact things.
        Test all permutations of where the approx and np.array() can show up
        """
        np = pytest.importorskip("numpy")
        expected = 100.0
        actual = 99.0
        abs_diff = expected - actual
        rel_diff = (expected - actual) / expected

        tests = [
            (eq, abs_diff, 0),
            (eq, 0, rel_diff),
            (ne, 0, rel_diff / 2.0),  # rel diff fail
            (ne, abs_diff / 2.0, 0),  # abs diff fail
        ]

        for op, _abs, _rel in tests:
            assert op(np.array(actual), approx(expected, abs=_abs, rel=_rel))  # a, b
            assert op(approx(expected, abs=_abs, rel=_rel), np.array(actual))  # b, a

            assert op(actual, approx(np.array(expected), abs=_abs, rel=_rel))  # a, b
            assert op(approx(np.array(expected), abs=_abs, rel=_rel), actual)  # b, a

            assert op(np.array(actual), approx(np.array(expected), abs=_abs, rel=_rel))
            assert op(approx(np.array(expected), abs=_abs, rel=_rel), np.array(actual))

    def test_numpy_expecting_nan(self):
        np = pytest.importorskip("numpy")
        examples = [
            (eq, nan, nan),
            (eq, -nan, -nan),
            (eq, nan, -nan),
            (ne, 0.0, nan),
            (ne, inf, nan),
        ]
        for op, a, x in examples:
            # Nothing is equal to NaN by default.
            assert np.array(a) != approx(x)
            assert a != approx(np.array(x))

            # If ``nan_ok=True``, then NaN is equal to NaN.
            assert op(np.array(a), approx(x, nan_ok=True))
            assert op(a, approx(np.array(x), nan_ok=True))

    def test_numpy_expecting_inf(self):
        np = pytest.importorskip("numpy")
        examples = [
            (eq, inf, inf),
            (eq, -inf, -inf),
            (ne, inf, -inf),
            (ne, 0.0, inf),
            (ne, nan, inf),
        ]
        for op, a, x in examples:
            assert op(np.array(a), approx(x))
            assert op(a, approx(np.array(x)))
            assert op(np.array(a), approx(np.array(x)))

    def test_numpy_array_wrong_shape(self):
        np = pytest.importorskip("numpy")

        a12 = np.array([[1, 2]])
        a21 = np.array([[1], [2]])

        assert a12 != approx(a21)
        assert a21 != approx(a12)

    def test_numpy_array_protocol(self):
        """
        array-like objects such as tensorflow's DeviceArray are handled like ndarray.
        See issue #8132
        """
        np = pytest.importorskip("numpy")

        class DeviceArray:
            def __init__(self, value, size):
                self.value = value
                self.size = size

            def __array__(self):
                return self.value * np.ones(self.size)

        class DeviceScalar:
            def __init__(self, value):
                self.value = value

            def __array__(self):
                return np.array(self.value)

        expected = 1
        actual = 1 + 1e-6
        assert approx(expected) == DeviceArray(actual, size=1)
        assert approx(expected) == DeviceArray(actual, size=2)
        assert approx(expected) == DeviceScalar(actual)
        assert approx(DeviceScalar(expected)) == actual
        assert approx(DeviceScalar(expected)) == DeviceScalar(actual)

    def test_doctests(self, mocked_doctest_runner) -> None:
        import doctest

        parser = doctest.DocTestParser()
        assert approx.__doc__ is not None
        test = parser.get_doctest(
            approx.__doc__, {"approx": approx}, approx.__name__, None, None
        )
        mocked_doctest_runner.run(test)

    def test_unicode_plus_minus(self, pytester: Pytester) -> None:
        """
        Comparing approx instances inside lists should not produce an error in the detailed diff.
        Integration test for issue #2111.
        """
        pytester.makepyfile(
            """
            import pytest
            def test_foo():
                assert [3] == [pytest.approx(4)]
        """
        )
        expected = "4.0e-06"
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [f"*At index 0 diff: 3 != 4 ± {expected}", "=* 1 failed in *="]
        )

    @pytest.mark.parametrize(
        "x, name",
        [
            pytest.param([[1]], "data structures", id="nested-list"),
            pytest.param({"key": {"key": 1}}, "dictionaries", id="nested-dict"),
        ],
    )
    def test_expected_value_type_error(self, x, name):
        with pytest.raises(
            TypeError,
            match=fr"pytest.approx\(\) does not support nested {name}:",
        ):
            approx(x)

    @pytest.mark.parametrize(
        "x",
        [
            pytest.param(None),
            pytest.param("string"),
            pytest.param(["string"], id="nested-str"),
            pytest.param({"key": "string"}, id="dict-with-string"),
        ],
    )
    def test_nonnumeric_okay_if_equal(self, x):
        assert x == approx(x)

    @pytest.mark.parametrize(
        "x",
        [
            pytest.param("string"),
            pytest.param(["string"], id="nested-str"),
            pytest.param({"key": "string"}, id="dict-with-string"),
        ],
    )
    def test_nonnumeric_false_if_unequal(self, x):
        """For nonnumeric types, x != pytest.approx(y) reduces to x != y"""
        assert "ab" != approx("abc")
        assert ["ab"] != approx(["abc"])
        # in particular, both of these should return False
        assert {"a": 1.0} != approx({"a": None})
        assert {"a": None} != approx({"a": 1.0})

        assert 1.0 != approx(None)
        assert None != approx(1.0)  # noqa: E711

        assert 1.0 != approx([None])
        assert None != approx([1.0])  # noqa: E711

    @pytest.mark.skipif(sys.version_info < (3, 7), reason="requires ordered dicts")
    def test_nonnumeric_dict_repr(self):
        """Dicts with non-numerics and infinites have no tolerances"""
        x1 = {"foo": 1.0000005, "bar": None, "foobar": inf}
        assert (
            repr(approx(x1))
            == "approx({'foo': 1.0000005 ± 1.0e-06, 'bar': None, 'foobar': inf})"
        )

    def test_nonnumeric_list_repr(self):
        """Lists with non-numerics and infinites have no tolerances"""
        x1 = [1.0000005, None, inf]
        assert repr(approx(x1)) == "approx([1.0000005 ± 1.0e-06, None, inf])"

    @pytest.mark.parametrize(
        "op",
        [
            pytest.param(operator.le, id="<="),
            pytest.param(operator.lt, id="<"),
            pytest.param(operator.ge, id=">="),
            pytest.param(operator.gt, id=">"),
        ],
    )
    def test_comparison_operator_type_error(self, op):
        """pytest.approx should raise TypeError for operators other than == and != (#2003)."""
        with pytest.raises(TypeError):
            op(1, approx(1, rel=1e-6, abs=1e-12))

    def test_numpy_array_with_scalar(self):
        np = pytest.importorskip("numpy")

        actual = np.array([1 + 1e-7, 1 - 1e-8])
        expected = 1.0

        assert actual == approx(expected, rel=5e-7, abs=0)
        assert actual != approx(expected, rel=5e-8, abs=0)
        assert approx(expected, rel=5e-7, abs=0) == actual
        assert approx(expected, rel=5e-8, abs=0) != actual

    def test_numpy_scalar_with_array(self):
        np = pytest.importorskip("numpy")

        actual = 1.0
        expected = np.array([1 + 1e-7, 1 - 1e-8])

        assert actual == approx(expected, rel=5e-7, abs=0)
        assert actual != approx(expected, rel=5e-8, abs=0)
        assert approx(expected, rel=5e-7, abs=0) == actual
        assert approx(expected, rel=5e-8, abs=0) != actual

    def test_generic_sized_iterable_object(self):
        class MySizedIterable:
            def __iter__(self):
                return iter([1, 2, 3, 4])

            def __len__(self):
                return 4

        expected = MySizedIterable()
        assert [1, 2, 3, 4] == approx(expected)
