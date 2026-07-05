# Testing: https://w3c.github.io/core-aam/#role-map-status

TEST_HTML = "<div role='status' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_STATUS_BAR
    # Object Attribute: container-live:polite
    # Object Attribute: live:polite
    # Object Attribute: container-live-role:status

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.STATUS_BAR
    assert "container-live:polite" in atspi.Accessible.get_attributes_as_array(node)
    assert "live:polite" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live-role:status" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXApplicationStatus

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_STATUSBAR
#     # Object Attribute: container-live:polite
#     # Object Attribute: live:polite
#     # Object Attribute: container-live-role:status

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Group
#     # Localized Control Type: status
#     # LiveSetting: Polite (1)
