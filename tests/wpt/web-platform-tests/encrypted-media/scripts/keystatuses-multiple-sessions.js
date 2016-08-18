function runTest(config)
{
    var testname = config.keysystem + ', keystatuses, multiple sessions';

    var configuration = getSimpleConfigurationForContent( config.content );

    if ( config.initDataType && config.initData ) configuration.initDataTypes = [ config.initDataType ];

    async_test(function(test)
    {
        var mediaKeySession1;
        var mediaKeySession2;

        // Even though key ids are uint8, using printable values so that
        // they can be verified easily.
        var key1 = new Uint8Array( config.content.keys[ 0 ].kid ),
            key2 = new Uint8Array( config.content.keys[ 1 ].kid );

        function processMessage1(event)
        {
            // This should only be called for session1.
            assert_equals(event.target, mediaKeySession1);

            // No keys added yet.
            verifyKeyStatuses(mediaKeySession1.keyStatuses, { expected: [], unexpected: [key1, key2] });

            // Add key1 to session1.
            config.messagehandler( config.keysystem, event.messageType, event.message ).then( function( response ) {

                event.target.update( response ).catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });
            });

        }

        function processKeyStatusesChange1(event)
        {
            // This should only be called for session1.
            assert_equals(event.target, mediaKeySession1);

            // Check that keyStatuses contains the expected key1 only.
            verifyKeyStatuses(mediaKeySession1.keyStatuses, { expected: [key1], unexpected: [key2] });

            // Now trigger a message event on session2.
            mediaKeySession2.generateRequest(config.initDataType, config.initData[ 1 ]).catch(function(error) {
                forceTestFailureFromPromise(test, error);
            });
        }

        function processMessage2(event)
        {
            // This should only be called for session2.
            assert_equals(event.target, mediaKeySession2);

            // session2 has no keys added yet.
            verifyKeyStatuses(mediaKeySession2.keyStatuses, { expected: [], unexpected: [key1, key2] });

            // session1 should still have 1 key.
            verifyKeyStatuses(mediaKeySession1.keyStatuses, { expected: [key1], unexpected: [key2] });

            // Add key2 to session2.
            config.messagehandler( config.keysystem, event.messageType, event.message ).then( function( response ) {

                event.target.update( response ).catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });
            });
        }

        function processKeyStatusesChange2(event)
        {
            // This should only be called for session2.
            assert_equals(event.target, mediaKeySession2);

            // Check that keyStatuses contains the expected key2 only.
            verifyKeyStatuses(mediaKeySession2.keyStatuses, { expected: [key2], unexpected: [key1] });

            // session1 should still have 1 key.
            verifyKeyStatuses(mediaKeySession1.keyStatuses, { expected: [key1], unexpected: [key2] });

            test.done();
        }

        navigator.requestMediaKeySystemAccess( config.keysystem, [ configuration ] ).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            mediaKeySession1 = mediaKeys.createSession();
            mediaKeySession2 = mediaKeys.createSession();

            // There should be no keys defined on either session.
            verifyKeyStatuses(mediaKeySession1.keyStatuses, { expected: [], unexpected: [key1, key2] });
            verifyKeyStatuses(mediaKeySession2.keyStatuses, { expected: [], unexpected: [key1, key2] });

            // Bind all the event handlers now.
            waitForEventAndRunStep('message', mediaKeySession1, processMessage1, test);
            waitForEventAndRunStep('message', mediaKeySession2, processMessage2, test);
            waitForEventAndRunStep('keystatuseschange', mediaKeySession1, processKeyStatusesChange1, test);
            waitForEventAndRunStep('keystatuseschange', mediaKeySession2, processKeyStatusesChange2, test);

            // Generate a request on session1.
            return mediaKeySession1.generateRequest(config.initDataType, config.initData[ 0 ] );
        }).catch(function(error) {
            forceTestFailureFromPromise(test, error);
        });
    },  testname );
}
