// META: script=/common/get-host-info.sub.js

async_test(function(t) {
    var channel1 = new MessageChannel();
    var channel2 = new MessageChannel();

     // First, transfer the port.
     channel2.port1.postMessage("1", [channel1.port1]);

     var counter = 0;
     channel1.port2.onmessage = t.step_func(function (evt) {
         if (counter == 0) {
             assert_equals(evt.data, "Start");
             counter = 1;
         } else {
             assert_equals(Number(evt.data), counter);
         }
         if (counter == 100) {
             t.done();
         }
         evt.target.postMessage(counter);
         counter = counter +1;
      });

      // Two, try using the just-transferred port.
      channel1.port1.postMessage("Should not be sent");

      channel2.port2.onmessage = t.step_func(function (evt) {
          evt.ports[0].postMessage("Start");
          evt.ports[0].onmessage = t.step_func(function (evt) {
              evt.target.postMessage(evt.data + 1);
          });
      });
}, "A port transferred to the same site is not garbage collected while active");

async_test(function(t) {
    var channel1 = new MessageChannel();
    var host = get_host_info();
    var noteSameSiteURL = host.HTTP_NOTSAMESITE_ORIGIN + "/webmessaging/support/ChildWindowPostMessage.htm";
    let iframe = document.createElement('iframe');
    iframe.src = noteSameSiteURL;
    document.body.appendChild(iframe);
    var TARGET = document.querySelector("iframe").contentWindow;
    iframe.onload = t.step_func(function() {
        // First, transfer the port.
        TARGET.postMessage("ports", "*", [channel1.port1]);

        var counter = 0;
        channel1.port2.postMessage(counter);
        channel1.port2.onmessage = t.step_func(function (evt) {
            assert_equals(Number(evt.data), counter);
            if (counter == 100) {
                t.done();
            }
            counter = counter +1;
            evt.target.postMessage(counter);
        });

        // Two, try using the just-transferred port.
        channel1.port1.postMessage("Should not be sent");
    });
}, "A port transferred to a different site is not garbage collected while active");
