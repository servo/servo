import pytest


@pytest.fixture
def dynamic():
    pass


@pytest.fixture
def a(request):
    request.getfixturevalue("dynamic")


@pytest.fixture
def b(a):
    pass


def test(b, request):
    assert request.fixturenames == ["b", "request", "a", "dynamic"]
