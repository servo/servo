function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem) + ', cannot load persistent license into temporary session';

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
            assert_in_array(event.messageType, ['license-request', 'individualization-request']);

            config.messagehandler(event.messageType, event.message).then( function(response) {
                mediaKeySession.update(response).then( test.step_func( function() {
                    if ( event.messageType !== 'license-request' ) {
                        return;
                    }
                    assert_unreached( "Update with incorrect license type should fail" )
                } ) ).catch( test.step_func( function( error ) {
                    if ( event.messageType !== 'license-request' ) {
                        forceTestFailureFromPromise(test, error);
                        return;
                    }

                    assert_equals(error.name, 'TypeError' );
                    test.done();
                } ) );
            }).catch(onFailure);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            initDataType = access.getConfiguration().initDataTypes[0];
            if (config.initDataType && config.initData) {
                initData = config.initData;
            } else {
                initData = getInitData(config.content, initDataType);
            }
            return access.createMediaKeys();
        }).then(test.step_func(function(mediaKeys) {
            mediaKeySession = mediaKeys.createSession('temporary');
            waitForEventAndRunStep('message', mediaKeySession, test.step_func(processMessage), test);
            return mediaKeySession.generateRequest(initDataType, initData);
        })).catch(onFailure);
    }, testname );

}
