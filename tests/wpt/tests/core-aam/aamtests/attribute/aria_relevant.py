# Testing: https://w3c.github.io/core-aam/#ariaRelevant

TEST_HTML = "<div role='group' id='test' aria-relevant='additions' aria-live='polite'> <div role='group' id='child'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: relevant:<value>
    # Object Attribute: container-relevant:<value>
    # Object Attribute: container-relevant:<value>: on all descendants

    node = atspi.find_node("test", session.url)
    assert "relevant:additions" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-relevant:additions" in atspi.Accessible.get_attributes_as_array(node)
    assert "container-relevant:additions" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIARelevant: <value>
#     # See also: Changes to document content or node visibility

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: relevant:<value>
#     # Object Attribute: container-relevant:<value>
#     # Object Attribute: container-relevant:<value>: on all descendants
#     # See also: Changes to document content or node visibility

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.relevant: <value>
#     # See also: Changes to document content or node visibility
