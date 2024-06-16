# mypy: allow-untyped-defs
import pytest


@pytest.fixture
def spam():
    return "spam"
