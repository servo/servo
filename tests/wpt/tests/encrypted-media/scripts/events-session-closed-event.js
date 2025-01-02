function runTest(config, qualifier)
{
    var testname = testnamePrefix(qualifier, config.keysystem) + ' test MediaKeySession closed event.';

    var configuration = {
        initDataTypes: [config.initDataType],
        audioCapabilities: [{
            contentType: config.audioType
        }],
        videoCapabilities: [{
            contentType: config.videoType
        }],
        sessionTypes: ['temporary']
    };

    promise_test(function (test) {
        var initDataType;
        var initData;
        var mediaKeySession;

        return navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function (access) {
            initDataType = access.getConfiguration().initDataTypes[0];
            return access.createMediaKeys();
        }).then(function (mediaKeys) {
            mediaKeySession = mediaKeys.createSession();
            if(config.initData) {
                initData = config.initData;
            } else {
                initData = stringToUint8Array(atob(config.content.keys[0].initData));
            }
            return mediaKeySession.generateRequest(initDataType, initData);
        }).then(function() {
            // close() should result in the closed promise being
            // fulfilled.
            return mediaKeySession.close();
        }).then(function (result) {
            assert_equals(result, undefined);
            // Wait for the session to be closed.
            return mediaKeySession.closed;
        }).then(function (result) {
            assert_equals(result, "closed-by-application");
            // Now that the session is closed, verify that the
            // closed attribute immediately returns a fulfilled
            // promise.
            return mediaKeySession.closed;
        }).then(function (result) {
            assert_equals(result, "closed-by-application");
        }).catch(function(error) {
            assert_unreached('Error: ' + error.name);
        });
    }, testname);
}