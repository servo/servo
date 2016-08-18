function runTest(config, testname) {

    var testname = config.keysystem + ', retrieve persistent-usage-record in new window, '
                                    + /video\/([^;]*)/.exec( config.videoType )[ 1 ];

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'persistent-usage-record' ] };


    async_test( function( test ) {

        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _sessionId;

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initDataType || event.initDataType,
                                                config.initData || event.initData ).then( function() {

                _sessionId = _mediaKeySession.sessionId;
            }).catch(function(error) {
                forceTestFailureFromPromise(test, error);
            });
        }

        function onMessage(event) {
            assert_equals( event.target, _mediaKeySession );
            assert_true( event instanceof window.MediaKeyMessageEvent );
            assert_equals( event.type, 'message');

            assert_in_array(  event.messageType, [ 'license-request', 'individualization-request' ] );

            config.messagehandler( config.keysystem, event.messageType, event.message ).then( function( response ) {

                _mediaKeySession.update( response )
                .catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });
            });
        }

        function onPlaying(event) {

            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        function onTimeupdate(event) {
            if ( _video.currentTime > ( config.duration || 5 ) ) {

                _video.removeEventListener('timeupdate', onTimeupdate );

                _video.pause();

                _mediaKeySession.closed.then( test.step_func( onClosed ) );

                _mediaKeySession.close();
            }
        }

        function onClosed(event) {

            _video.src = "";
            _video.setMediaKeys( null );

            var win = window.open( config.windowscript );
            window.addEventListener('message', test.step_func(function( event ) {

                event.data.forEach(test.step_func(function( assertion ) {

                    assert_equals(assertion.actual, assertion.expected, assertion.message);

                }));

                win.close();

                test.done();
            }));

            delete config.video;
            delete config.messagehandler;

            win.onload = function() {

                win.postMessage( { config: config, sessionId: _sessionId }, '*' );
            }
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [ configuration ]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;

            _video.setMediaKeys( mediaKeys );

            _mediaKeySession = _mediaKeys.createSession( 'persistent-usage-record' );

            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);
        }).then(function() {
            return testmediasource(config);
        }).then(function(source) {
            _mediaSource = source;
            _video.src = URL.createObjectURL(_mediaSource);
            _video.play();
        }).catch(function(error) {
            forceTestFailureFromPromise(test, error);
        });
    }, testname);
}