// META: script=/common/get-host-info.sub.js

async_test(function(t) {
    var host = get_host_info();
    var noteSameSiteURL = host.HTTP_NOTSAMESITE_ORIGIN + "/webmessaging/support/ChildWindowPostMessage.htm";
    var TOTALPORTS = 100;
    var LocalPorts = [];
    var RemotePorts = [];
    var PassedResult = 0;
    var sum = 0;
    let iframe = document.createElement('iframe');
    iframe.src = noteSameSiteURL;
    document.body.appendChild(iframe);
    var TARGET = document.querySelector("iframe").contentWindow;
    iframe.onload = t.step_func(function() {
        assert_own_property(window, "MessageChannel", "window");

        var channels = [];

        for (var i=0; i<TOTALPORTS; i++)
        {
            channels[i] = new MessageChannel();
            LocalPorts[i] = channels[i].port1;
            LocalPorts[i].foo = i;
            RemotePorts[i] = channels[i].port2;

            LocalPorts[i].onmessage = t.step_func(function(e)
            {
                assert_equals(e.target.foo, e.data);

                PassedResult++;
                sum += e.data;

                if (PassedResult == TOTALPORTS)
                {
                    assert_equals(sum, 4950);
                    t.done();
                }
            });
        }
        // Sending in two batches, to test the two postMessage variants.
        var firstBatch = RemotePorts.slice(0, 50);
        var secondBatch = RemotePorts.slice(50, 100);
        TARGET.postMessage("ports", "*", firstBatch);
        TARGET.postMessage("ports", {targetOrigin: '*', transfer: secondBatch});
    });
    var counter = 0;
    window.onmessage = function(e)
    {
        if (e.data === "ports")
        {
            if (counter == 0) {
                for (var i=0; i<51; i++)
                {
                    LocalPorts[i].postMessage(LocalPorts[i].foo);
                }
                counter = 1;
            } else {
                for (var i=51; i<100; i++)
                {
                    LocalPorts[i].postMessage(LocalPorts[i].foo);
                }
            }
        }
    }
}, "Test Description: postMessage to cross-site iframe with MessagePort array containing 100 ports.");
