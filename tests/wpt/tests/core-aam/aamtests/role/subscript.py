# Testing: https://w3c.github.io/core-aam/#role-map-subscript

TEST_HTML = "<div role='subscript' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SUBSCRIPT

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SUBSCRIPT

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXSubscriptStyleGroup

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_GROUPING
#     # Role: IA2_ROLE_TEXT_FRAME
#     # Text Attribute: text-position:sub

def test_uia(uia, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Control Type: Text
    # Styles used are exposed by IsSubscript attribute of the Text Control Pattern implemented on the accessible object.: IsSubscript: attribute of the TextRange Control Pattern implemented on the accessible object.

    node = uia.find_node("test", session.url)
    assert node.CurrentControlType == uia.ControlType.Text

    # Chrome exposes this as a textChild pattern, which is not a violation of UIA.
    assert (node.GetCurrentPropertyValue(uia.PropertyId.IsTextPatternAvailable) or node.GetCurrentPropertyValue(uia.PropertyId.IsTextChildPatternAvailable))

    text_child = node.GetCurrentPattern(uia.PatternId.TextChild)
    assert text_child is not None

    text_range = text_child.TextRange

    assert text_range.IsSubscript
    assert text_range.IsSuperscript == False
