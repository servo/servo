import pytest


@pytest.fixture
def order():
    return []


@pytest.fixture
def a(order):
    order.append("a")


@pytest.fixture
def b(a, order):
    order.append("b")


@pytest.fixture(autouse=True)
def c(b, order):
    order.append("c")


@pytest.fixture
def d(b, order):
    order.append("d")


@pytest.fixture
def e(d, order):
    order.append("e")


@pytest.fixture
def f(e, order):
    order.append("f")


@pytest.fixture
def g(f, c, order):
    order.append("g")


def test_order_and_g(g, order):
    assert order == ["a", "b", "c", "d", "e", "f", "g"]
