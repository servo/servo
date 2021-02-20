import pytest


@pytest.mark.parametrize("a", [r"qwe/\abc"])
def test_fixture(tmpdir, a):
    tmpdir.check(dir=1)
    assert tmpdir.listdir() == []
