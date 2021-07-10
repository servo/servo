
import py

def test_os():
    import os
    assert py.std.os is os

def test_import_error_converts_to_attributeerror():
    py.test.raises(AttributeError, "py.std.xyzalskdj")

def test_std_gets_it():
    for x in py.std.sys.modules:
        assert x in py.std.__dict__
