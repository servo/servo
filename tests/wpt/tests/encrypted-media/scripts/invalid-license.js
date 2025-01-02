function runTest(config)
{
    promise_test(function (test) {
        var initDataType;
        var initData;
        var keySystem = config.keysystem;
        var invalidLicense = new Uint8Array([0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
        var messageEventFired = false;

        var p = navigator.requestMediaKeySystemAccess(keySystem, getSimpleConfiguration()).then(function (access) {
            initDataType = access.getConfiguration().initDataTypes[0];
            initData = getInitData(initDataType);
            return access.createMediaKeys();
        }).then(function (mediaKeys) {
            var keySession = mediaKeys.createSession();
            var eventWatcher = new EventWatcher(test, keySession, ['message']);
            var promise = eventWatcher.wait_for('message');
            keySession.generateRequest(initDataType, initData);
            return promise;
        }).then(function (messageEvent) {
            messageEventFired = true;
            return messageEvent.target.update(invalidLicense);
        }).catch(function (error) {
            // Ensure we reached the update() call we are trying to test.
            if (!messageEventFired) {
                assert_unreached(
                    `Failed to reach the update() call.  Error: '${error.name}' '${error.message}'`);
            }

            // Propagate the error on through.
            throw error;
        });

        return promise_rejects_js(
            test, TypeError, p,
            'update() should fail because of an invalid license.');
    }, 'Update with invalid Clear Key license');
}
