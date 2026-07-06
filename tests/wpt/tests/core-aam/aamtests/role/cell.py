# Testing: https://w3c.github.io/core-aam/#role-map-cell

TEST_HTML = "<div role='table'> <div role='row'> <div role='cell' id='test'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_TABLE_CELL
    # Interface: TableCell

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.TABLE_CELL
    assert atspi.Accessible.get_table_cell(node) is not None

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXCell
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_CELL
#     # Interface: IAccessibleTableCell

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: DataItem
#     # Localized Control Type: item
#     # Control Pattern: GridItem
#     # Control Pattern: TableItem
