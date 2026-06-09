# Testing: https://w3c.github.io/core-aam/#role-map-separator

TEST_HTML = "<div role='separator' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SEPARATOR

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SEPARATOR

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXSplitter
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_SEPARATOR

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Separator
