// META: title=postMessage(): MessageEvent properties

    var TargetPort = null;
    var description = "The postMessage() method - Create an event that uses the MessageEvent interface, "
                    + "with the name message, which does not bubble and is not cancelable.";

    var t = async_test("Test Description: " + description);

    var channel = new MessageChannel();

    TargetPort = channel.port2;
    TargetPort.start();
    TargetPort.addEventListener("message", t.step_func(TestMessageEvent), true);

    channel.port1.postMessage("ping");

    function TestMessageEvent(evt)
    {
        ExpectedResult = [true, "message", false, false];
        ActualResult = [(evt instanceof MessageEvent), evt.type, evt.bubbles, evt.cancelable];

        assert_array_equals(ActualResult, ExpectedResult);
        t.done();
    }
