import sys

import pytest

from exceptiongroup import suppress

if sys.version_info < (3, 11):
    from exceptiongroup import BaseExceptionGroup, ExceptionGroup


def test_suppress_exception():
    with pytest.raises(ExceptionGroup) as exc, suppress(SystemExit):
        raise BaseExceptionGroup("", [SystemExit(1), RuntimeError("boo")])

    assert len(exc.value.exceptions) == 1
    assert isinstance(exc.value.exceptions[0], RuntimeError)
