function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem) + ', basic events';

    var configuration = getSimpleConfigurationForContent(config.content);

    if (config.initDataType && config.initData) {
        configuration.initDataTypes = [config.initDataType];
    }

    async_test(function(test)
    {
        var initDataType;
        var initData;
        var mediaKeySession;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function processMessage(event)
        {
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.target, mediaKeySession);
            assert_equals(event.type, 'message');
            assert_in_array(event.messageType,['license-request', 'individualization-request']);

            config.messagehandler( event.messageType, event.message ).then(function(response) {
                waitForEventAndRunStep('keystatuseschange', mediaKeySession, test.step_func(processKeyStatusesChange), test);
                return mediaKeySession.update( response );
            }).catch(onFailure);
        }

        function processKeyStatusesChange(event)
        {
            assert_true(event instanceof Event);
            assert_equals(event.target, mediaKeySession);
            assert_equals(event.type, 'keystatuseschange');
            test.done();
        }

        navigator.requestMediaKeySystemAccess(config.keysystem,[configuration]).then(function(access) {
            initDataType = access.getConfiguration().initDataTypes[0];

            if (config.initDataType && config.initData) {
                initData = config.initData;
            } else {
                initData = getInitData(config.content, initDataType);
            }

            return access.createMediaKeys();
        }).then(test.step_func(function(mediaKeys) {
            mediaKeySession = mediaKeys.createSession();
            waitForEventAndRunStep('message', mediaKeySession, test.step_func(processMessage), test);
            return mediaKeySession.generateRequest(initDataType, initData);
        })).catch(onFailure);
    }, testname );

}
