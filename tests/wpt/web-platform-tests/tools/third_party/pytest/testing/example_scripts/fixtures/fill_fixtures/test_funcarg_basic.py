# -*- coding: utf-8 -*-
import pytest


@pytest.fixture
def some(request):
    return request.function.__name__


@pytest.fixture
def other(request):
    return 42


def test_func(some, other):
    pass
