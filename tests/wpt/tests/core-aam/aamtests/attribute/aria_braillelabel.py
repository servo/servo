# Testing: https://w3c.github.io/core-aam/#ariaBraillelabel

TEST_HTML = "<button id='test' aria-braillelabel='foobar'> </button>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: braillelabel:<value>

    node = atspi.find_node("test", session.url)
    assert "braillelabel:foobar" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXBrailleLabel: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: braillelabel:<value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.braillelabel: <value>
