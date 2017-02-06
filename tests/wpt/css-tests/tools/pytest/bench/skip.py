
import pytest


SKIP = True

@pytest.mark.parametrize("x", xrange(5000))
def test_foo(x):
    if SKIP:
        pytest.skip("heh")
