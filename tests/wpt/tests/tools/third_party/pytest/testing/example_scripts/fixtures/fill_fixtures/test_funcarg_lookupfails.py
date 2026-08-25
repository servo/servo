# mypy: allow-untyped-defs
import pytest


@pytest.fixture
def xyzsomething(request):
    return 42


def test_func(some):
    pass
