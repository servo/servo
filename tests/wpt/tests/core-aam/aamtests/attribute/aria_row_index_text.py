# Testing: https://w3c.github.io/core-aam/#ariaRowIndexText

TEST_HTML = "<div role='grid'> <div role='row' id='row'> <div role='cell' id='test' aria-rowindextext='foo'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: rowindextext:<value>

    node = atspi.find_node("test", session.url)
    assert "rowindextext:foo" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXRowIndexDescription: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: rowindextext:<value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.rowindextext: <value>
