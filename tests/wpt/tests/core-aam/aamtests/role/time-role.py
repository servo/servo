# Testing: https://w3c.github.io/core-aam/#role-map-time

TEST_HTML = "<div role='time' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_STATIC
    # Object Attribute: xml-roles:time

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.STATIC
    assert "xml-roles:time" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXTimeGroup

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_GROUPING
#     # Object Attribute: xml-roles:time

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Text
#     # Localized Control Type: time
#     # Note: create a separate UIA Control of type Text. This is different from most UIA text mappings, which only create ranges in the page text pattern.
