import pytest

SKIP = True


@pytest.mark.parametrize("x", range(5000))
def test_foo(x):
    if SKIP:
        pytest.skip("heh")
