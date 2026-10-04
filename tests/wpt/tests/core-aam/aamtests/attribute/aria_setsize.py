# Testing: https://w3c.github.io/core-aam/#ariaSetsize

TEST_HTML = "<div role='list'> <div role='listitem' id='test' aria-setsize='3'>content</div> <div role='listitem' aria-setsize='3'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: setsize:<value>
    # State: STATE_INDETERMINATE: if the author-provided value is -1
    # If the author-provided value of aria-setsize is -1, the exposed value should be based on the number of objects in the DOM.

    node = atspi.find_node("test", session.url)
    assert "setsize:3" in atspi.Accessible.get_attributes_as_array(node)
    assert "STATE_INDETERMINATE" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIASetSize: <value>
#     # See also: Group Position

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: setsize:<value>
#     # See also: Group Position

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.setsize: <value>
#     # See also: Group Position
