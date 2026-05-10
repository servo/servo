# Testing: https://w3c.github.io/core-aam/#role-map-blockquote

TEST_HTML = "<div role='blockquote' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_BLOCK_QUOTE

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.BLOCK_QUOTE

def test_axapi(axapi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # AXRole: AXGroup
    # AXSubrole: <nil>

    node = axapi.find_node("test", session.url)
    role = axapi.AXUIElementCopyAttributeValue(node, "AXRole", None)[1]
    assert role == "AXGroup"
    role = axapi.AXUIElementCopyAttributeValue(node, "AXSubrole", None)[1]
    assert role == "AXUnknown"

def test_ia2(ia2, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SYSTEM_GROUPING
    # Role: IA2_ROLE_BLOCK_QUOTE

    node = ia2.find_node("test", session.url)
    assert ia2.get_role(node) == "IA2_ROLE_BLOCK_QUOTE"
    assert ia2.get_msaa_role(node) == "ROLE_SYSTEM_GROUPING"


# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#     node = uia.find_node("test", session.url)
#
#     # Spec:
#     # Control Type: Group
#     # Localized Control Type: blockquote
