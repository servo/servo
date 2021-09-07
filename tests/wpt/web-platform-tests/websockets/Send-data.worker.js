// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

importScripts("/resources/testharness.js");
importScripts('constants.sub.js')

var data = "test data";

async_test(function(t) {

    var wsocket = CreateWebSocket(false, false);

    wsocket.addEventListener('open', function (e) {
        wsocket.send(data)
    }, true)

    wsocket.addEventListener('message', t.step_func_done(function(e) {
            assert_equals(e.data, data);
    }), true);

    wsocket.addEventListener('close', t.unreached_func('the close event should not fire'), true);

}, "Send data on a WebSocket in a Worker")

done();
