def merge_dictionaries(first, second):
    """Given two dictionaries, create a third that defines all specified
    key/value pairs. This merge_dictionaries is performed "deeply" on any nested
    dictionaries. If a value is defined for the same key by both dictionaries,
    an exception will be raised."""
    result = dict(first)

    for key, value in second.items():
        if key in result and result[key] != value:
            if isinstance(result[key], dict) and isinstance(value, dict):
                result[key] = merge_dictionaries(result[key], value)
            elif result[key] != value:
                raise TypeError("merge_dictionaries: refusing to overwrite " +
                                  "attribute: `%s`" % key)
        else:
            result[key] = value

    return result

if __name__ == "__main__":
    assert merge_dictionaries({}, {}) == {}
    assert merge_dictionaries({}, {"a": 23}) == {"a": 23}
    assert merge_dictionaries({"a": 23}, {"b": 45}) == {"a": 23, "b": 45}

    e = None
    try:
        merge_dictionaries({"a": 23}, {"a": 45})
    except Exception as _e:
        e = _e
    assert isinstance(e, TypeError)

    assert merge_dictionaries({"a": 23}, {"a": 23}) == {"a": 23}

    assert merge_dictionaries({"a": {"b": 23}}, {"a": {"c": 45}}) == {"a": {"b": 23, "c": 45}}
    assert merge_dictionaries({"a": {"b": 23}}, {"a": {"b": 23}}) == {"a": {"b": 23}}

    e = None
    try:
        merge_dictionaries({"a": {"b": 23}}, {"a": {"b": 45}})
    except Exception as _e:
        e = _e
    assert isinstance(e, TypeError)
