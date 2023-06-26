function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', persistent-usage-record, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + 'playback';

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'persistent-usage-record' ] };


    async_test(function(test) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _releaseSequence = false;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            // event instance verification failing on CastTV
            // assert_true( event instanceof window.MediaKeyMessageEvent );
            assert_equals(event.type, 'message');

            if (!_releaseSequence)
            {
                assert_in_array(event.messageType, ['license-request', 'individualization-request']);
            }
            else
            {
                assert_equals(event.messageType, 'license-release');
            }

            config.messagehandler(event.messageType, event.message).then(function(response) {
                return _mediaKeySession.update(response);
            }).then(function() {
                if(event.messageType === 'license-request') {
                    return _video.setMediaKeys(_mediaKeys);
                } else if(event.messageType === 'license-release') {
                    test.done();
                }
            }).catch(onFailure);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initData ? config.initDataType : event.initDataType,
                                                config.initData || event.initData )
            .catch(onFailure);
        }

        function onClosed(event) {
            _video.src = "";
            _video.setMediaKeys( null );
        }

        function onTimeupdate(event) {
            if (_video.currentTime > ( config.duration || 1) && !_releaseSequence) {
                _video.removeEventListener('timeupdate', onTimeupdate );
                _video.pause();
                _releaseSequence = true;

                _mediaKeySession.closed.then(test.step_func(onClosed));
                _mediaKeySession.remove().catch(onFailure);

                _video.removeEventListener('timeupdate', onTimeupdate);
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
            _mediaKeySession = _mediaKeys.createSession('persistent-usage-record');
            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);
            return config.servercertificate ? _mediaKeys.setServerCertificate(config.servercertificate) : true;
        }).then(function(success) {
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
