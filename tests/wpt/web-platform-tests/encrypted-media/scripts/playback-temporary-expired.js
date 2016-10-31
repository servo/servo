function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', temporary, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', expired license';

    var configuration = {   initDataTypes: [config.initDataType],
                            audioCapabilities: [{contentType: config.audioType}],
                            videoCapabilities: [{contentType: config.videoType}],
                            sessionTypes: ['temporary'] };

    async_test(function(test) {

        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            // Only create the session for the first encrypted event
            if (_mediaKeySession !== undefined) return;

            var initDataType = config.initData ? config.initDataType : event.initDataType;
            var initData = config.initData || event.initData;

            _mediaKeySession = _mediaKeys.createSession('temporary');
            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(initDataType, initData).catch(onFailure);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array(event.messageType, ['license-request', 'individualization-request']);

            var expiration = Date.now().valueOf() + 1000;
            config.messagehandler(event.messageType, event.message, { expiration: expiration }).then(function(response) {
                return event.target.update(response);
            }).then(test.step_func(function() {
                // License server may only have second granularity, so check
                // that session expiration time is close to the desired value.
                assert_approx_equals(event.target.expiration, expiration, 2000, "expiration attribute should equal provided expiration time");

                // Since the expiration time is in the future, wait 5 seconds
                // so that the license has expired before calling play().
                test.step_timeout(function() {
                    assert_greater_than(Date.now().valueOf(), expiration, "Starting play before license expired");
                    _video.play();
                    // Wait 2 seconds to ensure that the video does not play.
                    test.step_timeout(function() { test.done(); }, 2000);
                }, 5000);
            })).catch(onFailure);
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', test.step_func(onTimeupdate), true);
        }

        function onTimeupdate(event) {
            _video.pause();
            assert_unreached("Playback should not start with expired license");
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            return _video.setMediaKeys(_mediaKeys);
        }).then(function(){
            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);
            return testmediasource(config);
        }).then(function(source) {
            _mediaSource = source;
            _video.src = URL.createObjectURL(_mediaSource);
            return source.done;
        }).catch(onFailure);
    }, testname);
}
