# SPDX-License-Identifier: MIT

# Keep this file SHORT, until Black can handle it.
import pytest

import attr


class TestPatternMatching:
    """
    Pattern matching syntax test cases.
    """

    @pytest.mark.parametrize("dec", [attr.s, attr.define, attr.frozen])
    def test_simple_match_case(self, dec):
        """
        Simple match case statement works as expected with all class
        decorators.
        """

        @dec
        class C(object):
            a = attr.ib()

        assert ("a",) == C.__match_args__

        matched = False
        c = C(a=1)
        match c:
            case C(a):
                matched = True

        assert matched
        assert 1 == a

    def test_explicit_match_args(self):
        """
        Does not overwrite a manually set empty __match_args__.
        """

        ma = ()

        @attr.define
        class C:
            a = attr.field()
            __match_args__ = ma

        c = C(a=1)

        msg = r"C\(\) accepts 0 positional sub-patterns \(1 given\)"
        with pytest.raises(TypeError, match=msg):
            match c:
                case C(_):
                    pass

    def test_match_args_kw_only(self):
        """
        kw_only classes don't generate __match_args__.
        kw_only fields are not included in __match_args__.
        """

        @attr.define
        class C:
            a = attr.field(kw_only=True)
            b = attr.field()

        assert ("b",) == C.__match_args__

        c = C(a=1, b=1)
        msg = r"C\(\) accepts 1 positional sub-pattern \(2 given\)"
        with pytest.raises(TypeError, match=msg):
            match c:
                case C(a, b):
                    pass

        found = False
        match c:
            case C(b, a=a):
                found = True

        assert found

        @attr.define(kw_only=True)
        class C:
            a = attr.field()
            b = attr.field()

        c = C(a=1, b=1)
        msg = r"C\(\) accepts 0 positional sub-patterns \(2 given\)"
        with pytest.raises(TypeError, match=msg):
            match c:
                case C(a, b):
                    pass

        found = False
        match c:
            case C(a=a, b=b):
                found = True

        assert found
        assert (1, 1) == (a, b)
