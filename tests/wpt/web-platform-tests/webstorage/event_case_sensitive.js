test(function() {
    var name ;
    testStorages(runTest);

    function runTest(storageString, callback)
    {
        name = storageString;
        window.completionCallback = callback;

        assert_true(storageString in window, storageString + " exist");
        window.storage = eval(storageString);

        storage.clear();
        assert_equals(storage.length, 0, "storage.length");
        storage.foo = "test";

        runAfterNStorageEvents(step1, 1);
    }

    function step1(msg)
    {
        storageEventList = new Array();
        storage.foo = "test";

        runAfterNStorageEvents(step2, 0);
    }

    function step2(msg)
    {
        test(function() {
            if(msg != undefined) {
                assert_unreached(msg);
            }
            assert_equals(storageEventList.length, 0);
        }, name + ": The key/value does not change, the event is not fired.");

        storage.foo = "TEST";

        runAfterNStorageEvents(step3, 1);
    }

    function step3(msg)
    {
        test(function() {
            if(msg != undefined) {
                assert_unreached(msg);
            }
            assert_equals(storageEventList.length, 1);
        }, name + ": The event is fired when the value case is changed.");

        completionCallback();
    }
}, "storage events fire even when only the case of the value changes.");

