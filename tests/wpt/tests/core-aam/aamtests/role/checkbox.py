# Testing: https://w3c.github.io/core-aam/#role-map-checkbox

TEST_HTML = "<div role='checkbox' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_CHECK_BOX

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.CHECK_BOX

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXCheckBox
#     # AXSubrole: <nil>
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_CHECKBUTTON
#     # See also: aria-checked in the State and Property Mapping Tables

def test_uia(uia, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Control Type: CheckBox
    # See also: aria-checked in the State and Property Mapping Tables

    node = uia.find_node("test", session.url)
    assert node.CurrentControlType == uia.ControlType.CheckBox

    assert node.GetCurrentPropertyValue(uia.PropertyId.IsTogglePatternAvailable)
    toggle_pattern = node.GetCurrentPattern(uia.PatternId.Toggle)
    assert toggle_pattern and toggle_pattern.CurrentToggleState == 0
