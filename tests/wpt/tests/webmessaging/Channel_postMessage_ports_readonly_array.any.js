// META: title=postMessage(): read-only ports array

    "use strict";

    var TargetPort = null;
    var description = "The postMessage() method - Make new ports into a read only array.";

    var t = async_test("Test Description: " + description);

    var channel = new MessageChannel();

    TargetPort = channel.port2;
    TargetPort.start();
    TargetPort.addEventListener("message", t.step_func(TestMessageEvent), true);

    var channel2 = new MessageChannel();

    channel.port1.postMessage("ports", [channel2.port1]);

    function TestMessageEvent(evt)
    {
        var channel3 = new MessageChannel();
        assert_throws_js(TypeError, () => {
            evt.ports.push(channel3.port1);
        }, "ports is a frozen object");
        assert_equals(evt.ports.length, 1, "ports is a read only array with length == 1.");
        t.done();
    }
