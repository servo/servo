function runTest(config) {

    var testname = config.keysystem + ', sucessful playback, temporary, '
                                    + /video\/([^;]*)/.exec( config.videoType )[ 1 ]
                                    + ', set src before setMediaKeys';

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    async_test( function( test ) {

        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource;

        function onMessage(event) {
            assert_equals( event.target, _mediaKeySession );
            assert_true( event instanceof window.MediaKeyMessageEvent );
            assert_equals( event.type, 'message');

            assert_any( assert_equals,
                        event.messageType,
                        [ 'license-request', 'individualization-request' ] );

            config.messagehandler( config.keysystem, event.messageType, event.message ).then( function( response ) {

                _mediaKeySession.update( response ).catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });
            });
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initData ? config.initDataType : event.initDataType,
                                                config.initData || event.initData )
            .catch(function(error) {
                forceTestFailureFromPromise(test, error);
            });

            _video.setMediaKeys(_mediaKeys);
        }

        function onTimeupdate(event) {
            if ( _video.currentTime > ( config.duration || 5 ) ) {
                _video.pause();
                test.done();
            }
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [ configuration ]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            _mediaKeySession = _mediaKeys.createSession( 'temporary' );

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