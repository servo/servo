# Testing: https://w3c.github.io/core-aam/#ariaHaspopupTree

TEST_HTML = "<div role='button' id='test' aria-haspopup='tree'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_HAS_POPUP
    # Object Attribute: haspopup:tree

    node = atspi.find_node("test", session.url)
    assert "STATE_HAS_POPUP" in atspi.get_state_list_helper(node)
    assert "haspopup:tree" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXPopupValue:tree
#     # Action: AXShowMenu

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_HASPOPUP
#     # Object Attribute: haspopup:tree

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Pattern: ExpandCollapse
#     # See also: aria-expanded
