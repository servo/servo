# Testing: https://w3c.github.io/core-aam/#role-map-document

TEST_HTML = "<div role='document' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_DOCUMENT_FRAME

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.DOCUMENT_FRAME

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXDocument

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_DOCUMENT
#     # State: STATE_SYSTEM_READONLY

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Document
