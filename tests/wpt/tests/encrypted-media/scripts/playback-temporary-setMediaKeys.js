SETMEDIAKEYS_IMMEDIATELY = 0;
SETMEDIAKEYS_AFTER_SRC = 1;
SETMEDIAKEYS_ONENCRYPTED = 2;
SETMEDIAKEYS_AFTER_UPDATE = 3;

function runTest(config,qualifier) {

    var testcase = (config.testcase === SETMEDIAKEYS_IMMEDIATELY) ? 'setMediaKeys first'
                    : (config.testcase === SETMEDIAKEYS_AFTER_SRC) ? 'setMediaKeys after setting video.src'
                    : (config.testcase === SETMEDIAKEYS_ONENCRYPTED) ? 'setMediaKeys in encrypted event'
                    : (config.testcase === SETMEDIAKEYS_AFTER_UPDATE) ? 'setMediaKeys after updating session'
                    : 'unknown';

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', temporary, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', playback, ' + testcase;

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    async_test(function(test) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array( event.messageType, ['license-request', 'individualization-request']);

            config.messagehandler(event.messageType, event.message).then(function(response) {
                return _mediaKeySession.update( response );
            }).then(function() {
                if (config.testcase === SETMEDIAKEYS_AFTER_UPDATE) {
                    return _video.setMediaKeys(_mediaKeys);
                }
            }).catch(onFailure);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            var promise = ( config.testcase === SETMEDIAKEYS_ONENCRYPTED )
                                ? _video.setMediaKeys(_mediaKeys)
                                : Promise.resolve();

            promise.then( function() {
                waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
                return _mediaKeySession.generateRequest(config.initData ? config.initDataType : event.initDataType,
                                                        config.initData || event.initData );
            }).catch(onFailure);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (config.duration || 1)) {
                _video.pause();
                test.done();
            }
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(test.step_func(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            if ( config.testcase === SETMEDIAKEYS_IMMEDIATELY ) {
                return _video.setMediaKeys( _mediaKeys );
            }
        })).then(function(){
            _mediaKeySession = _mediaKeys.createSession( 'temporary' );

            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);

            return testmediasource(config);
        }).then(function(source) {
            _mediaSource = source;
            _video.src = URL.createObjectURL(_mediaSource);
            return source.done;
        }).then(function(){
            _video.play();

            if (config.testcase === SETMEDIAKEYS_AFTER_SRC) {
                return _video.setMediaKeys(_mediaKeys);
            }
        }).catch(onFailure);
    }, testname);
}
