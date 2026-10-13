# Testing: https://w3c.github.io/core-aam/#ariaLiveOff

TEST_HTML = "<div role='group' id='test' aria-live='off'> <div role='group' id='child'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: live:off
    # Object Attribute: container-live:off
    # Object Attribute: container-live:off: on all descendants

    node = atspi.find_node("test", session.url)
    assert "live:off" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live:off" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-live:off" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIALive: "off"

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: live:off
#     # Object Attribute: container-live:off
#     # Object Attribute: container-live:off: on all descendants

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: LiveSetting: "off"
