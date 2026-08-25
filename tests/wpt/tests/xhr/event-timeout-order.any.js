// META: title=XMLHttpRequest: event - timeout (order of events)
// META: script=resources/xmlhttprequest-event-order.js

var test = async_test();
test.step(function () {
    var xhr = new XMLHttpRequest();
    prepare_xhr_for_event_order_test(xhr);
    xhr.addEventListener("loadend", function () {
        test.step(function () {
            assert_xhr_event_order_matches([1, "loadstart(0,0,false)", "upload.loadstart(0,12,true)", 4, "upload.timeout(0,0,false)", "upload.loadend(0,0,false)", "timeout(0,0,false)", "loadend(0,0,false)"]);
            test.done();
        });
    });

    xhr.timeout = 5;
    xhr.open("POST", "resources/delay.py?ms=20000");
    xhr.send("Test Message");
    test.step_timeout(() => {
        assert_unreached("ontimeout not called.");
    }, 2000);
});
