# Testing: https://w3c.github.io/core-aam/#ariaPosinset

TEST_HTML = "<div role='list'> <div role='listitem' id='test' aria-posinset='2'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: posinset:<value>

    node = atspi.find_node("test", session.url)
    assert "posinset:2" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIAPosInSet: <value>
#     # See also: Group Position

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: posinset:<value>
#     # See also: Group Position

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.posinset: <value>
#     # See also: Group Position
