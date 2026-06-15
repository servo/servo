# Testing: https://w3c.github.io/core-aam/#role-map-menuitemradio

TEST_HTML = "<div role='menu'> <div role='menuitemradio' id='test'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_RADIO_MENU_ITEM

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.RADIO_MENU_ITEM

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXMenuItem
#     # AXSubrole: <nil>
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_RADIOBUTTON: or ROLE_SYSTEM_MENUITEM
#     # Role: IA2_ROLE_RADIO_MENU_ITEM
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: MenuItem
#     # Control Pattern: Toggle
#     # Control Pattern: SelectionItem
#     # See also: aria-checked in the State and Property Mapping Tables
