# Testing: https://w3c.github.io/core-aam/#role-map-switch

TEST_HTML = "<div role='switch' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_TOGGLE_BUTTON
    # Object Attribute: xml-roles:switch

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.TOGGLE_BUTTON
    assert "xml-roles:switch" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXCheckBox
#     # AXSubrole: AXSwitch
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_CHECKBUTTON
#     # Role: IA2_ROLE_TOGGLE_BUTTON
#     # Object Attribute: xml-roles:switch
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Button
#     # Localized Control Type: toggleswitch
#     # Control Pattern: Toggle
#     # See also: aria-checked in the State and Property Mapping Tables
