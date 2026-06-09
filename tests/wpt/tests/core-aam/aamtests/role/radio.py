# Testing: https://w3c.github.io/core-aam/#role-map-radio

TEST_HTML = "<div role='radio' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_RADIO_BUTTON

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.RADIO_BUTTON

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXRadioButton
#     # AXSubrole: <nil>
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_RADIOBUTTON
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: RadioButton
#     # Control Pattern: Toggle
#     # Control Pattern: SelectionItem
#     # See also: aria-checked in the State and Property Mapping Tables
