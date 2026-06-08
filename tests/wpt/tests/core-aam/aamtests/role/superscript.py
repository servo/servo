# Testing: https://w3c.github.io/core-aam/#role-map-superscript

TEST_HTML = "<div role='superscript' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SUPERSCRIPT

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SUPERSCRIPT

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXSuperscriptStyleGroup

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_GROUPING
#     # Role: IA2_ROLE_TEXT_FRAME
#     # Text Attribute: text-position:super

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Text
#     # Styles used are exposed by IsSuperscript attribute of the TextRange Control Pattern implemented on the accessible object.: IsSuperscript: attribute of the TextRange Control Pattern implemented on the accessible object.
