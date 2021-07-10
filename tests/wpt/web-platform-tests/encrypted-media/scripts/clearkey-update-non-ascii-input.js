// This test is only applicable to clearkey
function runTest(config, qualifier)
{
    var testname = testnamePrefix(qualifier, config.keysystem) + ' test handling of non-ASCII responses for update()';

    var configuration = getSimpleConfigurationForContent(config.content);

    if (config.initDataType) {
        configuration.initDataTypes = [config.initDataType];
    }

    promise_test(function (test) {
        var initDataType;
        var initData;
        var mediaKeySession;
        var messageEventFired = false;

        var p = navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function (access) {
            initDataType = access.getConfiguration().initDataTypes[0];
            initData = getInitData(config.content, initDataType);
            return access.createMediaKeys();
        }).then(function (mediaKeys) {
            mediaKeySession = mediaKeys.createSession();
            var eventWatcher = new EventWatcher(test, mediaKeySession, ['message']);
            var promise = eventWatcher.wait_for('message');
            mediaKeySession.generateRequest(initDataType, initData);
            return promise;
        }).then(function (messageEvent) {
            // |jwkSet| contains a  non-ASCII character \uDC00.
            var jwkSet = '{"keys":[{'
                +     '"kty":"oct",'
                +     '"k":"MDEyMzQ1Njc4OTAxMjM0NQ",'
                +     '"kid":"MDEyMzQ1Njc4O\uDC00TAxMjM0NQ"'
                + '}]}';
            messageEventFired = true;
            return messageEvent.target.update(stringToUint8Array(jwkSet));
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
            'update() should fail because the processed message has non-ASCII character.');
    }, testname);
}
