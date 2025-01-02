# -*- coding: utf-8 -*-
"""
test_exceptions
~~~~~~~~~~~~~~~

Tests that verify logic local to exceptions.
"""
import h2.exceptions


class TestExceptions(object):
    def test_stream_id_too_low_prints_properly(self):
        x = h2.exceptions.StreamIDTooLowError(5, 10)

        assert "StreamIDTooLowError: 5 is lower than 10" == str(x)
