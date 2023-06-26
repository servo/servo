function runTest(config,qualifier) {

    // This test assumes one session is required for each provided initData

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', temporary, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', playback with multiple sessions, '
                                    + config.testcase;

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    async_test(function(test) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySessions = [],
            _mediaSource;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            assert_any(assert_equals, event.target, _mediaKeySessions);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array(event.messageType, ['license-request', 'individualization-request']);

            config.messagehandler(event.messageType, event.message, {variantId: event.target._variantId}).then(function(response) {
                return event.target.update(response);
            }).catch(onFailure);
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (config.duration || 1)) {
                _video.removeEventListener('timeupdate', onTimeupdate);
                _video.pause();
                test.done();
            }
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            return _video.setMediaKeys(_mediaKeys);
        }).then(function() {
            waitForEventAndRunStep('playing', _video, onPlaying, test);

            config.initData.forEach(function(initData,i) {
                var mediaKeySession = _mediaKeys.createSession( 'temporary' );
                mediaKeySession._variantId = config.variantIds ? config.variantIds[i] : undefined;
                waitForEventAndRunStep('message', mediaKeySession, onMessage, test);
                _mediaKeySessions.push(mediaKeySession);
                mediaKeySession.generateRequest(config.initDataType, initData).catch(onFailure);
            } );
            return testmediasource(config);
        }).then(function(source) {
            _mediaSource = source;
            _video.src = URL.createObjectURL(_mediaSource);
            return source.done;
        }).then(function(){
            _video.play();
        }).catch(onFailure);
    }, testname);
}
