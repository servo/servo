# Testing: https://w3c.github.io/core-aam/#role-map-menubar

TEST_HTML = "<div role='menubar' id='test'> <div role='menuitemradio'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_MENU_BAR
    # Interface: Selection
    # Because WAI-ARIA does not support modifying the selection via the accessibility API, user agents MUST return false for all Selection methods that provide a means to modify the selection.

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.MENU_BAR
    assert atspi.Accessible.get_selection_iface(node) is not None

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXMenuBar
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_MENUBAR
#     # Method: IAccessible::accSelect()
#     # Method: IAccessible::get_accSelection()

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: MenuBar
