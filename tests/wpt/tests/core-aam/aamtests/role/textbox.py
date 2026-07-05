# Testing: https://w3c.github.io/core-aam/#role-map-textbox

TEST_HTML = "<div role='textbox' contenteditable id='test'>content</div>"
TEST_HTML_READONLY = "<div role='textbox' contenteditable id='test' aria-readonly='true'>content</div>"

def test_atspi_editable(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_ENTRY
    # State: STATE_SINGLE_LINE
    # Interface: EditableText: if aria-readonly is not "true"

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.ENTRY
    #assert "STATE_SINGLE_LINE" in atspi.get_state_list_helper(node)
    assert atspi.Accessible.get_editable_text_iface(node) is not None


def test_atspi_readonly(atspi, session, inline):
    session.url = inline(TEST_HTML_READONLY)

    # Spec:
    # Role: ROLE_ENTRY
    # State: STATE_SINGLE_LINE
    # Interface: EditableText: if aria-readonly is not "true"

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.ENTRY
    #assert "STATE_SINGLE_LINE" in atspi.get_state_list_helper(node)
    assert atspi.Accessible.get_editable_text_iface(node) is None

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXTextField
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_TEXT
#     # State: IA2_STATE_SINGLE_LINE

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Edit
