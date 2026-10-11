# Testing: https://w3c.github.io/aria/html-aam/#el-abbr

TEST_HTML = "<abbr id='target' title='Accessibility API Mappings'>AAM</abbr>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ATK_ROLE_STATIC

    node = atspi.find_node("target", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.STATIC

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: (nil)
#     # AXRoleDescription: "group"

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Roles: ROLE_SYSTEM_TEXT; IA2_ROLE_TEXT_FRAME

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Text
