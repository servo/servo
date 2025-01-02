# mypy: allow-untyped-defs

def check_errors(errors):
    for e in errors:
        error_type, description, path, line_number = e
        assert isinstance(error_type, str)
        assert isinstance(description, str)
        assert isinstance(path, str)
        assert line_number is None or isinstance(line_number, int)
