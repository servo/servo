function runTest(config) {

    var testname = config.keysystem + ', sucessful playback and events, temporary, '
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
            _mediaSource,
            _allKeysUsableEvent = false,
            _playingEvent = false,
            _timeupdateEvent = false,
            _events = [ ];

        function onMessage(event) {
            assert_equals( event.target, _mediaKeySession );
            assert_true( event instanceof window.MediaKeyMessageEvent );
            assert_equals( event.type, 'message');

            assert_any( assert_equals,
                        event.messageType,
                        [ 'license-request', 'individualization-request' ] );

            if ( event.messageType !== 'individualization-request' ) {
                _events.push( event.messageType );
            }

            config.messagehandler( config.keysystem, event.messageType, event.message ).then( function( response ) {
                _events.push( 'license-response' );

                waitForEventAndRunStep('keystatuseschange', _mediaKeySession, onKeyStatusesChange, test);
                _mediaKeySession.update( response ).then( function() {
                    _events.push('updated');
                }).catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });
            });
        }

        function onKeyStatusesChange(event) {
            assert_equals(event.target, _mediaKeySession );
            assert_true(event instanceof window.Event );
            assert_equals(event.type, 'keystatuseschange' );

            var hasKeys = false, pendingKeys = false;
            _mediaKeySession.keyStatuses.forEach( function( value, keyid ) {
                assert_any( assert_equals, value, [ 'status-pending', 'usable' ] );

                hasKeys = true;
                pendingKeys = pendingKeys || ( value === 'status-pending' );

            });

            if ( !_allKeysUsableEvent && hasKeys && !pendingKeys ) {
                _allKeysUsableEvent = true;
                _events.push( 'allkeysusable' );
                _video.setMediaKeys(_mediaKeys);
            }

            if ( !hasKeys ) {
                _events.push( 'emptykeyslist' );
            }
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initData ? config.initDataType : event.initDataType,
                                                config.initData || event.initData ).then( function() {
                _events.push( 'generaterequest' );
            }).catch(function(error) {
                forceTestFailureFromPromise(test, error);
            });
        }

        function onClosed(event) {

            _events.push( 'closed-promise' );

            setTimeout( test.step_func( function() {

                assert_array_equals( _events,
                                    [
                                        'generaterequest',
                                        'license-request',
                                        'license-response',
                                        'updated',
                                        'allkeysusable',
                                        'playing',
                                        'closed',
                                        'closed-promise',
                                        'emptykeyslist'
                                    ],
                                    "Expected events sequence" );

                _video.src = "";
                _video.setMediaKeys( null ).then(function(){
                    test.done();
                });
            } ), 0 );
        }

        function onTimeupdate(event) {
            if ( _video.currentTime > ( config.duration || 5 ) && !_timeupdateEvent ) {
                _timeupdateEvent = true;
                _video.pause();

                _mediaKeySession.closed.then( test.step_func( onClosed ) );
                _mediaKeySession.close().then( function() {
                    _events.push( 'closed' );
                }).catch(function(error) {
                    forceTestFailureFromPromise(test, error);
                });
            }
        }

        function onPlaying(event) {
            _playingEvent = true;
            _events.push( 'playing' );

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