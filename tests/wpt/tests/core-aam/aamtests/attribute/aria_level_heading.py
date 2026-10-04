# Testing: https://w3c.github.io/core-aam/#ariaLevelHeading

TEST_HTML = "<div role='heading' id='test' aria-level='2'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: level:<value>

    node = atspi.find_node("test", session.url)
    assert "level:2" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValue: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: level:<value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.level: <value>
#     # Property: StyleId_Heading: <value>
