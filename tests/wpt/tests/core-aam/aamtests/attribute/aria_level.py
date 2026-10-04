# Testing: https://w3c.github.io/core-aam/#ariaLevel

TEST_HTML = "<div role='tree'> <div role='treeitem' id='test' aria-level='5'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: level:<value>

    node = atspi.find_node("test", session.url)
    assert "level:5" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXDisclosureLevel: <value> (zero-based), when used on an outline row (like a treeitem or group)

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: level:<value>
#     # Method: IAccessible2::groupPosition(): groupLevel=<value> on roles that support aria-posinset and aria-setsize
#     # See also: groupPosition()

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.level: <value>
