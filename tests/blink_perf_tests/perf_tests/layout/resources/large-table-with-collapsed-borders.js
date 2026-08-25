(function() {
    function createElement(tag, parent, className, id) {
        var el = document.createElement(tag);
        el.className = className;
        if (id)
            el.id = id;
        parent.appendChild(el);
        return el;
    }

    function createTable(width, height, colspan) {
        var table = createElement("table", document.body, "table");
        for (var y = 0; y < height; ++y) {
            var tr = createElement("tr", table, "tr");
            for (var x = 0; x < width; ++x) {
                var td = createElement("td", tr, "td");
                if (colspan > 0 && x==10 && y==0)
                    table.rows[y].cells[x].colSpan = colspan;
            }
        }
        return table;
    }

    function createTestFunction(width, height, colspan) {
        return function() {
            var table = createTable(width, height, colspan);
            PerfTestRunner.forceLayout();
        }
    }

    window.createTableTestFunction = createTestFunction;
})();
