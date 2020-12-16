# -*- coding: utf-8 -*-
import pytest


@pytest.fixture
def spam():
    return "spam"


class TestSpam(object):
    @pytest.fixture
    def spam(self, spam):
        return spam * 2

    def test_spam(self, spam):
        assert spam == "spamspam"
