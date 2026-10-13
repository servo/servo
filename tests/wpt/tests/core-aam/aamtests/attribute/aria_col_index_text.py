# Testing: https://w3c.github.io/core-aam/#ariaColIndexText

TEST_HTML = "<div role='grid'> <div role='row'> <div role='cell' id='test' aria-colindex='3' aria-colindextext='foo'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: colindextext:<value>

    node = atspi.find_node("test", session.url)
    assert "colindextext:foo" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXColumnIndexDescription: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: colindextext:<value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.colindextext: <value>
