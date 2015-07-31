var TestingUtils = (function() {

    function checkGridTemplateColumns(element, value) {
        assert_in_array(getComputedStyle(element).gridTemplateColumns, value, "gridTemplateColumns");
    }

    function checkGridTemplateRows(element, value) {
        assert_in_array(getComputedStyle(element).gridTemplateRows, value, "gridTemplateRows");
    }

    function testGridTemplateColumnsRows(gridId, columnsStyle, rowsStyle, columnsComputedValue, rowsComputedValue) {
        test(function() {
            var grid = document.getElementById(gridId);
            grid.style.gridTemplateColumns = columnsStyle;
            grid.style.gridTemplateRows = rowsStyle;
            checkGridTemplateColumns(grid, columnsComputedValue);
            checkGridTemplateRows(grid, rowsComputedValue);
        }, "'" + gridId + "' with: grid-template-columns: " + columnsStyle  + "; and grid-template-rows: " + rowsStyle + ";");
    }

    function testGridTemplateAreas(gridId, style, value) {
        test(function() {
            var grid = document.getElementById(gridId);
            grid.style.gridTemplateAreas = style;
            assert_equals(getComputedStyle(grid).gridTemplateAreas, value, "gridTemplateAreas");
        }, "'" + gridId + "' with: grid-template-areas: " + style + ";");
    }

    return {
        testGridTemplateColumnsRows: testGridTemplateColumnsRows,
        testGridTemplateAreas: testGridTemplateAreas
    }
})();
