// META: title=postMessage() with a host object raises DataCloneError

    var description = "Throw a DataCloneError when a host object (e.g. a DOM node) is used with postMessage.";

    test(function()
    {
        var channel = new MessageChannel();
        channel.port1.start();

        assert_throws_dom("DATA_CLONE_ERR", function()
        {
            channel.port1.postMessage(globalThis);
        });
    }, description);
