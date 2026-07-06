# Testing: https://w3c.github.io/core-aam/#role-map-tab

TEST_HTML = "<div role='tablist'> <div role='tab' id='test'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_PAGE_TAB
    # State: STATE_SELECTED: if focus is inside tabpanel associated with aria-labelledby (TODO: Add test)

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.PAGE_TAB

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXRadioButton
#     # AXSubrole: AXTabButton

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_PAGETAB
#     # State: STATE_SYSTEM_SELECTED: if focus is inside tabpanel associated with aria-labelledby

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: TabItem
