# Testing: https://w3c.github.io/core-aam/#ariaBrailleroledescriptionUndefined

TEST_HTML = "<button id='test' aria-brailleroledescription=''> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Not mapped

    node = atspi.find_node("test", session.url)
    assert 'brailleroledescription' not in atspi.Accessible.get_attributes(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped
