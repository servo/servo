from _pytest.compat import importlib_metadata


def test_pytest_entry_points_are_identical():
    dist = importlib_metadata.distribution("pytest")
    entry_map = {ep.name: ep for ep in dist.entry_points}
    assert entry_map["pytest"].value == entry_map["py.test"].value
