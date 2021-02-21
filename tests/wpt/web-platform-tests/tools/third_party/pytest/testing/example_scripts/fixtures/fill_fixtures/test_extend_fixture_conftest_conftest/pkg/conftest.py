import pytest


@pytest.fixture
def spam(spam):
    return spam * 2
