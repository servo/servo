# Testing: https://w3c.github.io/core-aam/#role-map-suggestion

TEST_HTML = "<div role='suggestion' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SUGGESTION
    # Object Attribute: xml-roles:suggestion

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SUGGESTION
    assert "xml-roles:suggestion" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXAttributedStringForTextMarkerRange: AXIsSuggestion = 1;: for all text contained in a suggestion

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: IA2_ROLE_SUGGESTION
#     # Object Attribute: xml-roles:suggestion

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Group
#     # Localized Control Type: suggestion
