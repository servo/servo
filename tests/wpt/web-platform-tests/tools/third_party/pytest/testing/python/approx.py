# encoding: utf-8
import operator
import sys
import pytest
import doctest

from pytest import approx
from operator import eq, ne
from decimal import Decimal
from fractions import Fraction

inf, nan = float("inf"), float("nan")


class MyDocTestRunner(doctest.DocTestRunner):

    def __init__(self):
        doctest.DocTestRunner.__init__(self)

    def report_failure(self, out, test, example, got):
        raise AssertionError(
            "'{}' evaluates to '{}', not '{}'".format(
                example.source.strip(), got.strip(), example.want.strip()
            )
        )


class TestApprox(object):

    def test_repr_string(self):
        plus_minus = u"\u00b1" if sys.version_info[0] > 2 else u"+-"
        tol1, tol2, infr = "1.0e-06", "2.0e-06", "inf"
        assert repr(approx(1.0)) == "1.0 {pm} {tol1}".format(pm=plus_minus, tol1=tol1)
        assert (
            repr(approx([1.0, 2.0]))
            == "approx([1.0 {pm} {tol1}, 2.0 {pm} {tol2}])".format(
                pm=plus_minus, tol1=tol1, tol2=tol2
            )
        )
        assert (
            repr(approx((1.0, 2.0)))
            == "approx((1.0 {pm} {tol1}, 2.0 {pm} {tol2}))".format(
                pm=plus_minus, tol1=tol1, tol2=tol2
            )
        )
        assert repr(approx(inf)) == "inf"
        assert repr(approx(1.0, rel=nan)) == "1.0 {pm} ???".format(pm=plus_minus)
        assert (
            repr(approx(1.0, rel=inf))
            == "1.0 {pm} {infr}".format(pm=plus_minus, infr=infr)
        )
        assert repr(approx(1.0j, rel=inf)) == "1j"

        # Dictionaries aren't ordered, so we need to check both orders.
        assert repr(approx({"a": 1.0, "b": 2.0})) in (
            "approx({{'a': 1.0 {pm} {tol1}, 'b': 2.0 {pm} {tol2}}})".format(
                pm=plus_minus, tol1=tol1, tol2=tol2
            ),
            "approx({{'b': 2.0 {pm} {tol2}, 'a': 1.0 {pm} {tol1}}})".format(
                pm=plus_minus, tol1=tol1, tol2=tol2
            ),
        )

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

    def test_negative_tolerance(self):
        # Negative tolerances are not allowed.
        illegal_kwargs = [
            dict(rel=-1e100),
            dict(abs=-1e100),
            dict(rel=1e100, abs=-1e100),
            dict(rel=-1e100, abs=1e100),
            dict(rel=-1e100, abs=-1e100),
        ]
        for kwargs in illegal_kwargs:
            with pytest.raises(ValueError):
                1.1 == approx(1, **kwargs)

    def test_inf_tolerance(self):
        # Everything should be equal if the tolerance is infinite.
        large_diffs = [(1, 1000), (1e-50, 1e50), (-1.0, -1e300), (0.0, 10)]
        for a, x in large_diffs:
            assert a != approx(x, rel=0.0, abs=0.0)
            assert a == approx(x, rel=inf, abs=0.0)
            assert a == approx(x, rel=0.0, abs=inf)
            assert a == approx(x, rel=inf, abs=inf)

    def test_inf_tolerance_expecting_zero(self):
        # If the relative tolerance is zero but the expected value is infinite,
        # the actual tolerance is a NaN, which should be an error.
        illegal_kwargs = [dict(rel=inf, abs=0.0), dict(rel=inf, abs=inf)]
        for kwargs in illegal_kwargs:
            with pytest.raises(ValueError):
                1 == approx(0, **kwargs)

    def test_nan_tolerance(self):
        illegal_kwargs = [dict(rel=nan), dict(abs=nan), dict(rel=nan, abs=nan)]
        for kwargs in illegal_kwargs:
            with pytest.raises(ValueError):
                1.1 == approx(1, **kwargs)

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

    def test_dict_wrong_len(self):
        assert {"a": 1, "b": 2} != approx({"a": 1})
        assert {"a": 1, "b": 2} != approx({"a": 1, "c": 2})
        assert {"a": 1, "b": 2} != approx({"a": 1, "b": 2, "c": 3})

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

    def test_numpy_array_wrong_shape(self):
        np = pytest.importorskip("numpy")

        a12 = np.array([[1, 2]])
        a21 = np.array([[1], [2]])

        assert a12 != approx(a21)
        assert a21 != approx(a12)

    def test_doctests(self):
        parser = doctest.DocTestParser()
        test = parser.get_doctest(
            approx.__doc__, {"approx": approx}, approx.__name__, None, None
        )
        runner = MyDocTestRunner()
        runner.run(test)

    def test_unicode_plus_minus(self, testdir):
        """
        Comparing approx instances inside lists should not produce an error in the detailed diff.
        Integration test for issue #2111.
        """
        testdir.makepyfile(
            """
            import pytest
            def test_foo():
                assert [3] == [pytest.approx(4)]
        """
        )
        expected = "4.0e-06"
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            ["*At index 0 diff: 3 != 4 * {}".format(expected), "=* 1 failed in *="]
        )

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
        """
        pytest.approx should raise TypeError for operators other than == and != (#2003).
        """
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
