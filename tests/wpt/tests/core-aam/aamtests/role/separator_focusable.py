# Testing: https://w3c.github.io/core-aam/#role-map-separator-focusable

TEST_HTML = "<div role='separator' tabindex='0' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SEPARATOR
    # Interface: Value
    # Because WAI-ARIA does not support modifying the value via the accessibility API, user agents MUST return false for all Value methods that provide a means to modify the value.

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SEPARATOR
    assert atspi.Accessible.get_value_iface(node) is not None

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
#     # Interface: IAccessibleValue

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Thumb
#     # Control Pattern: RangeValue
