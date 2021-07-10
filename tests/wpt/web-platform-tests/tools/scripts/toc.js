// grab the table of contents filled with all the anchors
function __result_handler() {
    function getMap() {
        var toc_element = document.getElementById("contents").nextElementSibling;

        function getSection() {
            function getIds(node) {
                var a = [];

                var nodes = node.querySelectorAll('*[id]');
                for (var i = 0; i < nodes.length; i++) {
                    a.push(nodes[i].getAttribute("id"));
                }
                return a;
            }

            function getTOCIds() {
                var a = [];

                var nodes = toc_element.querySelectorAll('li');
                for (var i = 0; i < nodes.length; i++) {
                    var href = nodes[i].firstElementChild.getAttribute("href");
                    a.push(href.substring(1));
                }
                return a;
            }

            var obj = new Object();
            var ids = getIds(document);
            var toc = getTOCIds();

            for (var i = 1; i < toc.length; i++) {
                var key1 = toc[i-1];
                var key2 = toc[i];
                var map = [];

                var index1 = ids.indexOf(key1);
                var index2 = ids.indexOf(key2);

                if ((index2-index1) > 1) {
                    for (var j = index1+1; j < index2;j++) {
                        map.push(ids[j]);
                    }
                }

                obj[key1] = map;
            }
            {
                var key = toc[toc.length-1];
                var index = ids.indexOf(key);
                var map = [];

                for (var j = index+1; j < ids.length;j++) {
                    map.push(ids[j]);
                }
                obj[key] = map;
            }

            return obj;
        }

        function section(id) {
            this.id = id;
        }
        function addSubSection(section, sub) {
            if (typeof section.sections === "undefined") {
                section.sections = [];
            }
            section.sections.push(sub);
        }

        function li(el, map) {
            var obj = new section(el.firstElementChild.getAttribute("href").substring(1));
            obj.title = el.firstElementChild.textContent;
            var child = el.firstElementChild;

            var m = map[obj.id];
            for (var i = 0; i < m.length; i++)  {
                var sub = new section(m[i]);
                addSubSection(obj, sub);
            }
            while (child !== null) {
                if (child.nodeName === "OL") ol(child, obj, map);
                child = child.nextElementSibling;
            }
            return obj;
        }

        function ol(el, section, map) {
            var child = el.firstElementChild;
            while (child !== null) {
                addSubSection(section, li(child, map));
                child = child.nextElementSibling;
            }
        }

        var map = getSection();
        var main = new section("___main___");
        main.title = document.title;

        ol(toc_element, main, map);

        return main;
    }

    return getMap();
}
