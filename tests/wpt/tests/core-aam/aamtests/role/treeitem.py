# Testing: https://w3c.github.io/core-aam/#role-map-treeitem

TEST_HTML = "<div role='tree'> <div role='treeitem' id='test'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_TREE_ITEM

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.TREE_ITEM

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXRow
#     # AXSubrole: AXOutlineRow
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_OUTLINEITEM
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: TreeItem
#     # See also: aria-checked in the State and Property Mapping Tables
