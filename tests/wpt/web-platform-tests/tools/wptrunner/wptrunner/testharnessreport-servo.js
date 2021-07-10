var props = {output:%(output)d, debug: %(debug)s};
var start_loc = document.createElement('a');
start_loc.href = location.href;
setup(props);

add_completion_callback(function (tests, harness_status) {
    var id = decodeURIComponent(start_loc.pathname) + decodeURIComponent(start_loc.search) + decodeURIComponent(start_loc.hash);
    console.log("ALERT: RESULT: " + JSON.stringify([
        id,
        harness_status.status,
        harness_status.message,
        harness_status.stack,
        tests.map(function(t) {
            return [t.name, t.status, t.message, t.stack]
        }),
    ]));
});
