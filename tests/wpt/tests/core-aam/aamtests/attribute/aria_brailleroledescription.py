# Testing: https://w3c.github.io/core-aam/#ariaBrailleroledescription

TEST_HTML = "<button id='test' aria-brailleroledescription='foobar'> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: brailleroledescription:<value>

    node = atspi.find_node("test", session.url)
    assert "brailleroledescription:foobar" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: brailleroledescription:<value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.brailleroledescription: <value>
