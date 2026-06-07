# Testing: https://w3c.github.io/core-aam/#role-map-insertion

TEST_HTML = "<div role='insertion' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_CONTENT_INSERTION
    # Object Attribute: xml-roles:insertion

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.CONTENT_INSERTION
    assert "xml-roles:insertion" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXInsertStyleGroup
#     # AXAttributedStringForTextMarkerRange: AXIsSuggestedInsertion = 1;: for all text contained in a insertion

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: IA2_ROLE_CONTENT_INSERTION

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Text
#     # Localized Control Type: insertion
