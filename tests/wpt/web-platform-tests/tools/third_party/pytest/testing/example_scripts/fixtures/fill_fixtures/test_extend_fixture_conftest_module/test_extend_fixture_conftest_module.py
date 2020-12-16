# -*- coding: utf-8 -*-
import pytest


@pytest.fixture
def spam(spam):
    return spam * 2


def test_spam(spam):
    assert spam == "spamspam"
