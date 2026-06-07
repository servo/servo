# Testing: https://w3c.github.io/core-aam/#role-map-subscript

TEST_HTML = "<div role='subscript' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SUBSCRIPT

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SUBSCRIPT

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXSubscriptStyleGroup

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_GROUPING
#     # Role: IA2_ROLE_TEXT_FRAME
#     # Text Attribute: text-position:sub

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Text
#     # Styles used are exposed by IsSubscript attribute of the TextRange Control Pattern implemented on the accessible object.: IsSubscript: attribute of the TextRange Control Pattern implemented on the accessible object.
