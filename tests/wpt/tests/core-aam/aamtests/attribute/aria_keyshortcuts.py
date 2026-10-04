# Testing: https://w3c.github.io/core-aam/#ariaKeyshortcuts

TEST_HTML = "<div role='group' id='test' aria-keyshortcuts='Shift+Space'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: keyshortcuts:<value>

    node = atspi.find_node("test", session.url)
    assert "keyshortcuts:Shift+Space" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXKeyShortcutsValue: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: accKeyboardShortcut: <value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AcceleratorKey: <value>
