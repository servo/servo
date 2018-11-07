importScripts("/resources/testharness.js");
importScripts('websocket.sub.js')

var data = "test data";

async_test(function(t) {

    var wsocket = CreateWebSocket(false, false, false);

    wsocket.addEventListener('open', function (e) {
        wsocket.send(data)
    }, true)

    wsocket.addEventListener('message', t.step_func_done(function(e) {
            assert_equals(e.data, data);
            done();
    }), true);

}, "Send data on a WebSocket in a Worker")


