# Testing: https://w3c.github.io/core-aam/#role-map-listbox

TEST_HTML = "<div role='listbox' id='test'> <div role='option'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_LIST_BOX
    # Interface: Selection
    # Because WAI-ARIA does not support modifying the selection via the accessibility API, user agents MUST return false for all Selection methods that provide a means to modify the selection.

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.LIST_BOX
    assert atspi.Accessible.get_selection_iface(node) is not None

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXList
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_LIST
#     # Method: IAccessible::accSelect()
#     # Method: IAccessible::get_accSelection()

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: List
#     # Control Pattern: Selection
