// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

[
  [0, "0"],
  [500, "500"],
  [NaN, "NaN"],
  ["string", "String"],
  [null, "null"],
  [0x10000 + 1000, "2**16+1000"],
].forEach(function(t) {
  test(function() {
    var ws = CreateWebSocket(false, false);
    assert_throws_dom("InvalidAccessError", function() {
      ws.close(t[0]);
    });
    ws.onerror = this.unreached_func();
  }, t[1] + " on a websocket");
});
