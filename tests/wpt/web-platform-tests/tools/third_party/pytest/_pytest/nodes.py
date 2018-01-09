SEP = "/"


def _splitnode(nodeid):
    """Split a nodeid into constituent 'parts'.

    Node IDs are strings, and can be things like:
        ''
        'testing/code'
        'testing/code/test_excinfo.py'
        'testing/code/test_excinfo.py::TestFormattedExcinfo::()'

    Return values are lists e.g.
        []
        ['testing', 'code']
        ['testing', 'code', 'test_excinfo.py']
        ['testing', 'code', 'test_excinfo.py', 'TestFormattedExcinfo', '()']
    """
    if nodeid == '':
        # If there is no root node at all, return an empty list so the caller's logic can remain sane
        return []
    parts = nodeid.split(SEP)
    # Replace single last element 'test_foo.py::Bar::()' with multiple elements 'test_foo.py', 'Bar', '()'
    parts[-1:] = parts[-1].split("::")
    return parts


def ischildnode(baseid, nodeid):
    """Return True if the nodeid is a child node of the baseid.

    E.g. 'foo/bar::Baz::()' is a child of 'foo', 'foo/bar' and 'foo/bar::Baz', but not of 'foo/blorp'
    """
    base_parts = _splitnode(baseid)
    node_parts = _splitnode(nodeid)
    if len(node_parts) < len(base_parts):
        return False
    return node_parts[:len(base_parts)] == base_parts
