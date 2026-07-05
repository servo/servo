# Testing: https://w3c.github.io/core-aam/#role-map-mark

TEST_HTML = "<div role='mark' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_MARK
    # Object Attribute: xml-roles:mark

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.MARK
    assert "xml-roles:mark" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXRoleDescription: highlight
#     # AXAttributedStringForTextMarkerRange: AXHighlight = 1;: for all text contained in a mark

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_GROUPING
#     # Role: IA2_ROLE_MARK
#     # Object Attribute: xml-roles:mark

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Group
