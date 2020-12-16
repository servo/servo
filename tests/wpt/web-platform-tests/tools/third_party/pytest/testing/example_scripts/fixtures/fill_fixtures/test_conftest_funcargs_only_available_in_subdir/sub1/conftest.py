# -*- coding: utf-8 -*-
import pytest


@pytest.fixture
def arg1(request):
    with pytest.raises(Exception):
        request.getfixturevalue("arg2")
