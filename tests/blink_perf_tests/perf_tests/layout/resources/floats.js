(function() {
    function createElement(tag, parent, className, id) {
        var el = document.createElement(tag);
        el.className = className;
        if (id)
            el.id = id;
        parent.appendChild(el);
        return el;
    }

    function createSet(width, height, nested) {
        var container = createElement("div", document.body, "container");
        for (var y = 0; y < height; ++y) {
            for (var x = 0; x < width; ++x)
                createElement("div", container, "float", "float" + x + "_" + y);

            var nestedContainer = container;
            for ( ; nested > 0; --nested)
                nestedContainer = createElement("div", nestedContainer, "nested", "nested" + x + "_" + nested);
            
            createElement("div", container, "float-end", "end" + x)
        }
        return container;
    }

    function toggle(str, str1, str2) {
        return str == str1 ? str2 : str1;
    }
    
    function resetTest() {
        PerfTestRunner.resetRandomSeed();
        var list = document.querySelectorAll(".float.big");
        for (var i = 0; i < list.length; ++i)
            list[i].className = "float";
    }
    
    function createTestFunction(width, height, nested, runs, rows) {
        var containers = [];
        for (var i = 0; i < rows; ++i)
            containers[i] = createSet(width, height, nested);
        nested = nested || 0;
        runs = runs || 10;
        return function() {
            for (var c = 0; c < rows; ++c) {
                container = containers[c];
                container.style.display = "block";
                for (var i = 0; i < runs; ++i) {
                    var x = Math.floor(Math.random() * width);
                    var y = Math.floor(Math.random() * height);
                    var el = document.getElementById("float" + x + "_" + y);
                    el.className = toggle(el.className, "float", "float big");
                    PerfTestRunner.forceLayout();
                }
                resetTest();
                container.style.display = "none";
            }
        }
    }
    
    window.createFloatsLayoutTestFunction = createTestFunction;
})();
