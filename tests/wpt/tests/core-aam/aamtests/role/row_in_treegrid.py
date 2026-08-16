# Testing: https://w3c.github.io/core-aam/#role-map-row-in-treegrid

TEST_HTML = "<div role='treegrid'> <div role='row' id='test'> <div role='gridcell'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_TABLE_ROW

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.TABLE_ROW

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXRow
#     # AXSubrole: <nil>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_OUTLINEITEM

def test_uia(uia, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Control Type: DataItem
    # Localized Control Type: row
    # Control Pattern: SelectionItem

    node = uia.find_node("test", session.url)
    assert node.CurrentControlType == uia.ControlType.DataItem
    assert node.CurrentLocalizedControlType == "row"

    assert node.GetCurrentPropertyValue(uia.PropertyId.IsSelectionItemPatternAvailable)
    selection_pattern = node.GetCurrentPattern(uia.PatternId.SelectionItem)
    assert selection_pattern and selection_pattern.CurrentIsSelected == 0
