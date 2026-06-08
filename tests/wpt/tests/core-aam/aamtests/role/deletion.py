# Testing: https://w3c.github.io/core-aam/#role-map-deletion

TEST_HTML = "<div role='deletion' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_CONTENT_DELETION
    # Object Attribute: xml-roles:deletion

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.CONTENT_DELETION
    assert "xml-roles:deletion" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXDeleteStyleGroup
#     # AXAttributedStringForTextMarkerRange: AXIsSuggestedDeletion = 1;: for all text contained in a deletion

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: IA2_ROLE_CONTENT_DELETION

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Text
#     # Localized Control Type: deletion
