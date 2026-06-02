function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem) + ', cannot load persistent license into temporary session';

    var configuration = getSimpleConfigurationForContent(config.content);

    if (config.initDataType && config.initData) {
        configuration.initDataTypes = [config.initDataType];
    }

    // Check if DRM server for key system config supports persistent license.
    var supportsPersistentLicense = false;
    if (drmconfig[config.keysystem]) {
        supportsPersistentLicense = drmconfig[config.keysystem].some(function(cfg) {
            return cfg.sessionTypes !== undefined && cfg.sessionTypes.indexOf('persistent-license') !== -1;
        });
    }

    // Skip test if persistent license is not supported.
    if (!supportsPersistentLicense) {
        test(function() {
            assert_true(true, "DRM server for key system not configured for persistent license, test skipped");
        }, testname);
        return;
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

                    assert_throws_js(TypeError, () => { throw error; });
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
