// META: script=/common/get-host-info.sub.js

async_test(function(t) {
  var channel1 = new MessageChannel();
  var host = get_host_info();
  let iframe = document.createElement('iframe');
  iframe.src = host.HTTP_NOTSAMESITE_ORIGIN + "/webmessaging/support/ChildWindowPostMessage.htm";
  document.body.appendChild(iframe);
  var TARGET = document.querySelector("iframe").contentWindow;
  iframe.onload = t.step_func(function() {
    // Enable the port.
    channel1.port1.onmessage = t.step_func(function (evt) {
      assert_equals(Number(evt.data), 0);

      // Send a message, expecting it to be received in the iframe.
      channel1.port2.postMessage(1)

      // Transfer the port.
      TARGET.postMessage("ports", "*", [channel1.port1]);
    });

    // Send a message, expecting it to be received here.
    channel1.port2.postMessage(0)

    channel1.port2.onmessage = t.step_func(function (evt) {
        assert_equals(Number(evt.data), 1);
        t.done();
      });
  });
}, `Tasks enqueued on the port-message-queue of an enabled port,
  are transferred along with the port, when the transfer happens in the same task
  during which postMessage is called`);
