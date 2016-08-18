function runTest(config) {

    var testname = config.keysystem + ', sucessful playback, temporary, '
                                    + /video\/([^;]*)/.exec( config.videoType )[ 1 ]
                                    + ', set src before setMediaKeys';

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    promise_test(function(test)
    {
        var promises = config.video.map( function( video ) { return play_video_as_promise( test, video ); } );

        return Promise.all(promises);

    }, testname );

    function play_video_as_promise( test, _video ) {
        var _mediaKeys,
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

        function wait_for_timeupdate_message(video)
        {
            return new Promise(function(resolve) {
                video.addEventListener('timeupdate', function listener(event) {
                    if ( event.target.currentTime > ( config.duration || 5 ) )
                    {
                        video.removeEventListener('timeupdate', listener);
                        resolve(event);
                    }
                });
            });
        };

        return navigator.requestMediaKeySystemAccess(config.keysystem, [ configuration ]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            _mediaKeySession = _mediaKeys.createSession( 'temporary' );

            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);

        }).then(function() {
            return testmediasource(config);
        }).then(function(source) {
            _mediaSource = source;
            _video.src = URL.createObjectURL(_mediaSource);
            _video.play();
            return wait_for_timeupdate_message(_video);
        }).catch(function(error) {
            forceTestFailureFromPromise(test, error);
        });
    }
}