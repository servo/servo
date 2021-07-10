function checkSelectionAttributes(anchorNode, anchorOffset, focusNode, focusOffset, collapsed, rangeCount)
{
    var selection = window.getSelection();
    assert_equals(selection.anchorNode, anchorNode, "anchorNode");
    assert_equals(selection.anchorOffset, anchorOffset, "anchorOffset");
    assert_equals(selection.focusNode, focusNode, "focusNode");
    assert_equals(selection.focusOffset, focusOffset, "focusOffset");
    assert_equals(selection.isCollapsed, collapsed, "collapsed");
    assert_equals(selection.rangeCount, rangeCount, "rangeCount");
}

function checkDefaultSelectionAttributes()
{
    checkSelectionAttributes(null, 0, null, 0, true, 0);
}
