# Testing: https://w3c.github.io/core-aam/#role-map-gridcell

TEST_HTML = "<div role='grid'> <div role='row'> <div role='gridcell' id='test'>content</div> </div> </div>"

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

def test_uia(uia, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Control Type: DataItem
    # Localized Control Type: item
    # Control Pattern: GridItem
    # Control Pattern: TableItem
    # Control Pattern: SelectionItem
    # SelectionItem.SelectionContainer: grid

    node = uia.find_node("test", session.url)
    assert node.CurrentControlType == uia.ControlType.DataItem
    assert node.CurrentLocalizedControlType == "item"
    assert node.GetCurrentPropertyValue(uia.PropertyId.IsGridItemPatternAvailable)
    assert node.GetCurrentPropertyValue(uia.PropertyId.IsTableItemPatternAvailable)

    assert node.GetCurrentPropertyValue(uia.PropertyId.IsSelectionPatternAvailable)
    selection_container = node.GetCurrentPattern(uia.PatternId.SelectionItem).CurrentSelectionContainer
    assert selection_container.CurrentControlType == uia.ControlType.DataGrid
