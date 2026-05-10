// META: script=/common/get-host-info.sub.js

async_test(function(t) {
  var channel1 = new MessageChannel();
  var channel2 = new MessageChannel();
  var host = get_host_info();
  let iframe = document.createElement('iframe');
  iframe.src = host.HTTP_NOTSAMESITE_ORIGIN + "/webmessaging/support/ChildWindowPostMessage.htm";
  document.body.appendChild(iframe);
  var TARGET = document.querySelector("iframe").contentWindow;
  iframe.onload = t.step_func(function() {

    // Send a message, expecting it to be received in the iframe.
    channel1.port2.postMessage(1)

    // First, transfer the port into the same realm.
    channel2.port2.postMessage(0, [channel1.port1]);

    channel2.port1.onmessage = t.step_func(function (evt) {
      assert_equals(Number(evt.data), 0);

      t.step_timeout(function () {
        // Transfer the port to the iframe.
        TARGET.postMessage("ports", "*", evt.ports);

        // Keep the event-loop busy for one second,
        // which will result in the iframe
        // starting the "complete port transfer" flow,
        // before the window global could finish it's own.
        var request = new XMLHttpRequest();
        request.open('GET', 'blank.html?pipe=trickle(d1)', false);
        request.send(null);
      }, 0);
    });

    channel1.port2.onmessage = t.step_func(function (evt) {
      assert_equals(Number(evt.data), 1);
      t.done();
    });
  });
}, `A port transferred outside of a onmessage handler,
  followed by a delay in returning the buffer caused by blocking the event-loop,
  does not lose messages along the way.`);
