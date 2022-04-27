import pytest


@pytest.mark.parametrize("a", [r"qwe/\abc"])
def test_fixture(tmp_path, a):
    assert tmp_path.is_dir()
    assert list(tmp_path.iterdir()) == []
