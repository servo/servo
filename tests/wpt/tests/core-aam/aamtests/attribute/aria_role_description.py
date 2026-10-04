# Testing: https://w3c.github.io/core-aam/#ariaRoleDescription

TEST_HTML = "<div role='group' id='test' aria-roledescription='hello world'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: roledescription:<value>

    node = atspi.find_node("test", session.url)
    assert "roledescription:hello world" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXRoleDescription: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: localizedExtendedRole(): <value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Localized Control Type: <value>
