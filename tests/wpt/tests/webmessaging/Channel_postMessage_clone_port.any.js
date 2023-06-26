// META: title=postMessage(): clone a port

    var OriginalPort = null;
    var ClonedPort = null;
    var description = "Test Description: When the user agent is to clone a port original port, with "
                    + "the clone being owned by owner, it must return a new MessagePort object";

    var t = async_test("Test Description: " + description);

    var ChannelA = new MessageChannel();
    var ChannelB = new MessageChannel();
    OriginalPort = ChannelB.port2;

    ChannelA.port2.onmessage = t.step_func(function(evt)
    {
        if(evt.data == "ports")
        {
            ClonedPort = evt.ports[0];

            assert_not_equals(ClonedPort, OriginalPort, "new cloned port object should not equal to the original port!");

            ClonedPort.onmessage = function(e)
            {
                test(function(){ assert_equals(e.data, "ping"); }, "Data sent through remote port is received by the new cloned port");
                t.done();
            }
        }
    });

    ChannelA.port1.postMessage("ports", [OriginalPort]);
    ChannelB.port1.postMessage("ping");
