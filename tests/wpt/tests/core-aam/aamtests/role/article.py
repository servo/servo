# Testing: https://w3c.github.io/core-aam/#role-map-article

TEST_HTML = "<div role='article' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_ARTICLE
    # Object Attribute: xml-roles:article

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.ARTICLE
    assert "xml-roles:article" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXDocumentArticle

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_DOCUMENT
#     # State: STATE_SYSTEM_READONLY
#     # Object Attribute: xml-roles:article

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Group
#     # Localized Control Type: article
