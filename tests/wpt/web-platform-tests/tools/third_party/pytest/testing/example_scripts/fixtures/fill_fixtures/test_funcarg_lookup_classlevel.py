# -*- coding: utf-8 -*-
import pytest


class TestClass(object):
    @pytest.fixture
    def something(self, request):
        return request.instance

    def test_method(self, something):
        assert something is self
