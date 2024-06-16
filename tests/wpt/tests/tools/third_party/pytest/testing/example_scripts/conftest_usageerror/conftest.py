# mypy: allow-untyped-defs
def pytest_configure(config):
    import pytest

    raise pytest.UsageError("hello")


def pytest_unconfigure(config):
    print("pytest_unconfigure_called")
