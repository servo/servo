# Testing: https://w3c.github.io/core-aam/#role-map-log

TEST_HTML = "<div role='log' id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_LOG
    # Object Attribute: xml-roles:log
    # Object Attribute: container-live:polite
    # Object Attribute: live:polite
    # Object Attribute: container-live-role:log

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.LOG
    assert "xml-roles:log" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live:polite" in atspi.Accessible.get_attributes_as_array(node)
    assert "live:polite" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live-role:log" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXApplicationLog

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: xml-roles:log
#     # Object Attribute: container-live:polite
#     # Object Attribute: live:polite
#     # Object Attribute: container-live-role:log

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Group
#     # Localized Control Type: log
#     # LiveSetting: Polite (1)
