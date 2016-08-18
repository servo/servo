function runTest(config, testname) {

    var testname = config.keysystem + ', successful playback, persistent-usage-record, '
                                    + /video\/([^;]*)/.exec( config.videoType )[ 1 ]
                                    + ', set src before setMediaKeys';

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'persistent-usage-record' ] };


    async_test( function( test ) {

        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _releaseSequence = false;

        function onMessage(event) {
            assert_equals( event.target, _mediaKeySession );
            assert_true( event instanceof window.MediaKeyMessageEvent );
            assert_equals( event.type, 'message');

            if ( !_releaseSequence )
            {
                assert_in_array( event.messageType, [ 'license-request', 'individualization-request' ] );
            }
            else
            {
                assert_equals( event.messageType, 'license-release' );
            }

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
        }

        function onClosed(event) {
            _video.src = "";
            _video.setMediaKeys( null ).then(function(){
                    test.done();
            });
        }

        function onTimeupdate(event) {
            if ( _video.currentTime > ( config.duration || 5 ) && !_releaseSequence ) {

                _video.removeEventListener('timeupdate', onTimeupdate );

                _video.pause();

                _releaseSequence = true;

                _mediaKeySession.closed.then( test.step_func( onClosed ) );
                _mediaKeySession.remove().catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });

                _video.removeEventListener('timeupdate', onTimeupdate );
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
            _video.setMediaKeys(_mediaKeys);
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