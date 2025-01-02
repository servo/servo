function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem) + ', expiration';

    var configuration = getSimpleConfigurationForContent(config.content);
    if (config.initDataType && config.initData) {
        configuration.initDataTypes = [config.initDataType];
    }

    async_test(function(test) {

        var _mediaKeys,
            _mediaKeySession;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array(event.messageType, [ 'license-request', 'individualization-request' ] );

            config.messagehandler(event.messageType, event.message, {expiration: config.expiration}).then(function(response) {
                return event.target.update(response);
            }).then(test.step_func(function() {
                assert_approx_equals(event.target.expiration, config.expiration, 4000, "expiration attribute should equal provided expiration time");
                test.done();
            })).catch(onFailure);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            _mediaKeySession = _mediaKeys.createSession( 'temporary' );
            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            return _mediaKeySession.generateRequest(config.initDataType, config.initData);
        }).catch(onFailure);
    }, testname);
}
