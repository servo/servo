# Testing: https://w3c.github.io/core-aam/#role-map-superscript

TEST_HTML = "<div role='superscript' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_SUPERSCRIPT

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.SUPERSCRIPT

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXSuperscriptStyleGroup

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_GROUPING
#     # Role: IA2_ROLE_TEXT_FRAME
#     # Text Attribute: text-position:super

def test_uia(uia, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Control Type: Text
    # Styles used are exposed by IsSuperscript attribute of the TextRange Control Pattern implemented on the accessible object.: IsSuperscript: attribute of the TextRange Control Pattern implemented on the accessible object.

    node = uia.find_node("test", session.url)
    assert node.CurrentControlType == uia.ControlType.Text

    # Chrome exposes this as a TextChild pattern, which is not a violation of UIA.
    assert (node.GetCurrentPropertyValue(uia.PropertyId.IsTextPatternAvailable) or node.GetCurrentPropertyValue(uia.PropertyId.IsTextChildPatternAvailable))

    text_child = node.GetCurrentPattern(uia.PatternId.TextChild)
    assert text_child is not None

    text_range = text_child.TextRange

    assert text_range.IsSuperscript
    assert text_range.IsSubscript == False