# Testing: https://w3c.github.io/core-aam/#ariaLiveAssertive

TEST_HTML = "<div role='group' id='test' aria-live='assertive'> <div role='group' id='child'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: live:assertive
    # Object Attribute: container-live:assertive
    # Object Attribute: container-live:assertive: on all descendants

    node = atspi.find_node("test", session.url)
    assert "live:assertive" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live:assertive" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live:assertive" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIALive: "assertive"
#     # See also: Changes to document content or node visibility

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: live:assertive
#     # Object Attribute: container-live:assertive
#     # Object Attribute: container-live:assertive: on all descendants
#     # See also: Changes to document content or node visibility

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: LiveSetting: "assertive"
#     # See also: Changes to document content or node visibility
