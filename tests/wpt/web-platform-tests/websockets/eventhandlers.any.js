// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

function testEventHandler(name) {
  test(function() {
    var ws = CreateWebSocket(true, false);
    assert_equals(ws["on" + name], null);
    ws["on" + name] = function() {};
    ws["on" + name] = 2;
    assert_equals(ws["on" + name], null);
  }, "Event handler for " + name + " should have [TreatNonCallableAsNull]")
}
["open", "error", "close", "message"].forEach(testEventHandler);
