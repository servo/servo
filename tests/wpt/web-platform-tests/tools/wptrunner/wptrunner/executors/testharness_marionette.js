window.wrappedJSObject.timeout_multiplier = %(timeout_multiplier)d;
window.wrappedJSObject.explicit_timeout = %(explicit_timeout)d;

window.wrappedJSObject.addEventListener("message", function listener(event) {
    if (event.data.type != "complete") {
        return;
    }
    window.wrappedJSObject.removeEventListener("message", listener);
    clearTimeout(timer);
    var tests = event.data.tests;
    var status = event.data.status;

    var subtest_results = tests.map(function (x) {
        return [x.name, x.status, x.message, x.stack]
    });

    marionetteScriptFinished(["%(url)s",
                              status.status,
                              status.message,
                              status.stack,
                              subtest_results]);
}, false);

window.wrappedJSObject.win = window.open("%(abs_url)s", "%(window_id)s");

var timer = null;
if (%(timeout)s) {
    timer = setTimeout(function() {
        window.wrappedJSObject.win.timeout();
    }, %(timeout)s);
}
