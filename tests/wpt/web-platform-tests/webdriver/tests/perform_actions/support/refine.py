def get_events(session):
    """Return list of key events recorded in the test_keys_page fixture."""
    events = session.execute_script("return allEvents.events;") or []
    # `key` values in `allEvents` may be escaped (see `escapeSurrogateHalf` in
    # test_keys_wdspec.html), so this converts them back into unicode literals.
    for e in events:
        # example: turn "U+d83d" (6 chars) into u"\ud83d" (1 char)
        if "key" in e and e["key"].startswith(u"U+"):
            key = e["key"]
            hex_suffix = key[key.index("+") + 1:]
            e["key"] = unichr(int(hex_suffix, 16))

        # WebKit sets code as 'Unidentified' for unidentified key codes, but
        # tests expect ''.
        if "code" in e and e["code"] == "Unidentified":
            e["code"] = ""
    return events


def get_keys(input_el):
    """Get printable characters entered into `input_el`.

    :param input_el: HTML input element.
    """
    rv = input_el.property("value")
    if rv is None:
        return ""
    else:
        return rv
