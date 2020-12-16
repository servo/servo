# -*- coding: utf-8 -*-
import pytest


@pytest.fixture
def fix1(fix2):
    return 1


@pytest.fixture
def fix2(fix1):
    return 1


def test(fix1):
    pass
