// META: script=websocket.sub.js

[
  [0, "0"],
  [500, "500"],
  [NaN, "NaN"],
  ["string", "String"],
  [null, "null"],
  [0x10000 + 1000, "2**16+1000"],
].forEach(function(t) {
  [true, false].forEach(function(secure) {
    test(function() {
      var ws = CreateWebSocket(secure, false, false);
      assert_throws_dom("InvalidAccessError", function() {
        ws.close(t[0]);
      });
      wsocket.onerror = this.unreached_func();
    }, t[1] + " on a " + (secure ? "secure" : "insecure") + " websocket");
  });
});
