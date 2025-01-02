// META: title=basic messagechannel with transfer

async_test(function(t) {
  var channel = new MessageChannel();
  var ab = new ArrayBuffer(1);
  channel.port1.postMessage(ab, {transfer: [ab]});
  channel.port2.onmessage = t.step_func(
    function(e) {
      assert_equals(e.data.byteLength, 1);
      t.done();
    });
  channel.port2.start();
});
