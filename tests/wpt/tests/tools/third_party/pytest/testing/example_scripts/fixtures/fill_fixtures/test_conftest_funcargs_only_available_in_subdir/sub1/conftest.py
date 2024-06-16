# mypy: allow-untyped-defs
import pytest


@pytest.fixture
def arg1(request):
    with pytest.raises(pytest.FixtureLookupError):
        request.getfixturevalue("arg2")
