import pytest


@pytest.fixture(scope="module", params=range(966))
def foo(request):
    return request.param


def test_it(foo):
    pass


def test_it2(foo):
    pass
