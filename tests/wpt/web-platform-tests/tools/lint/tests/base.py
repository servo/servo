from __future__ import unicode_literals

from six import integer_types, text_type

def check_errors(errors):
    for e in errors:
        error_type, description, path, line_number = e
        assert isinstance(error_type, text_type)
        assert isinstance(description, text_type)
        assert isinstance(path, text_type)
        assert line_number is None or isinstance(line_number, integer_types)
