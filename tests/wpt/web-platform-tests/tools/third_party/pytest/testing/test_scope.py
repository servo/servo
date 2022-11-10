import re

import pytest
from _pytest.scope import Scope


def test_ordering() -> None:
    assert Scope.Session > Scope.Package
    assert Scope.Package > Scope.Module
    assert Scope.Module > Scope.Class
    assert Scope.Class > Scope.Function


def test_next_lower() -> None:
    assert Scope.Session.next_lower() is Scope.Package
    assert Scope.Package.next_lower() is Scope.Module
    assert Scope.Module.next_lower() is Scope.Class
    assert Scope.Class.next_lower() is Scope.Function

    with pytest.raises(ValueError, match="Function is the lower-most scope"):
        Scope.Function.next_lower()


def test_next_higher() -> None:
    assert Scope.Function.next_higher() is Scope.Class
    assert Scope.Class.next_higher() is Scope.Module
    assert Scope.Module.next_higher() is Scope.Package
    assert Scope.Package.next_higher() is Scope.Session

    with pytest.raises(ValueError, match="Session is the upper-most scope"):
        Scope.Session.next_higher()


def test_from_user() -> None:
    assert Scope.from_user("module", "for parametrize", "some::id") is Scope.Module

    expected_msg = "for parametrize from some::id got an unexpected scope value 'foo'"
    with pytest.raises(pytest.fail.Exception, match=re.escape(expected_msg)):
        Scope.from_user("foo", "for parametrize", "some::id")  # type:ignore[arg-type]
