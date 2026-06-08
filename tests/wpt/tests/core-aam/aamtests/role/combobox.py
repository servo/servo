# Testing: https://w3c.github.io/core-aam/#role-map-combobox

TEST_HTML = "<div role='combobox' aria-expanded='false' id='test'> <div role='textbox'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_COMBO_BOX
    # State: STATE_EXPANDABLE
    # State: STATE_HAS_POPUP

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.COMBO_BOX
    assert "STATE_EXPANDABLE" in atspi.get_state_list_helper(node)
    assert "STATE_HAS_POPUP" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXComboBox
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_COMBOBOX
#     # State: STATE_SYSTEM_HASPOPUP
#     # State: STATE_SYSTEM_COLLAPSED: if aria-expanded is not "true"

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Combobox
