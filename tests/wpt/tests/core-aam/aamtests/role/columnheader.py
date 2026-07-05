# Testing: https://w3c.github.io/core-aam/#role-map-columnheader

TEST_HTML = "<div role='grid'> <div role='row'> <div role='columnheader' id='test'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_COLUMN_HEADER
    # Interface: TableCell

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.COLUMN_HEADER
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
#     # Role: ROLE_SYSTEM_COLUMNHEADER
#     # Interface: IAccessibleTableCell

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: DataItem
#     # Localized Control Type: column header
#     # Control Pattern: GridItem
#     # Control Pattern: TableItem
