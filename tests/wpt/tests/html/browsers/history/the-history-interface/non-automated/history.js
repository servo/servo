function parse_query() {
    var query = location.search.slice(1);
    var vars = query.split("&");
    var fields = {};
    vars.forEach(
        function (x) {
            var split = x.split("=");
            return fields[split[0]] = split.slice(1).join("=");
        });
    return fields;
}

var query_parts = parse_query();
var id = "id" in query_parts ? parseInt(query_parts.id) : 1;
var urls_to_load = query_parts.urls.split(",");

document.write(id);

onunload = function() {};

function queue_next() {
    t = opener.t;
    setTimeout(t.step_func(
        function() {
//            opener.assert_equals(history.length, id);
            if (urls_to_load[0]) {
                var next_page = urls_to_load[0];
                (next_page.indexOf("?") > -1) ? (next_page += "&") : (next_page += "?");
                next_page += "urls=" + urls_to_load.slice(1).join(",");
                next_page += "&id=" + ++id;
                location = next_page;
            }
        }
    ), 100);
}
