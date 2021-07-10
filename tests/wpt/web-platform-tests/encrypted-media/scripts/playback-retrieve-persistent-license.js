function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', persistent-license, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', ' + config.testcase;

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'persistent-license' ] };


    async_test( function(test) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _sessionId;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initData ? config.initDataType : event.initDataType,
                                                config.initData || event.initData ).then( function() {
                _sessionId = _mediaKeySession.sessionId;
            }).catch(onFailure);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array(event.messageType, ['license-request', 'individualization-request']);

            config.messagehandler(event.messageType, event.message).then(function(response) {
                return _mediaKeySession.update(response);
            }).catch(onFailure);
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (config.duration || 1)) {
                _video.removeEventListener('timeupdate', onTimeupdate);
                _video.pause();
                _video.removeAttribute('src');
                _video.load();

                _mediaKeySession.closed
                    .then(test.step_func(onClosed))
                    .catch(onFailure);
                _mediaKeySession.close()
                    .catch(onFailure);
            }
        }

        function onClosed() {
            // Open a new window in which we will attempt to play with the persisted license
            var win = window.open(config.windowscript);
            assert_not_equals(win, null, "Popup windows not allowed?");

            // Lisen for an event from the new window containing its test assertions
            window.addEventListener('message', test.step_func(function(messageEvent) {
                if (messageEvent.data.testResult) {
                    messageEvent.data.testResult.forEach(test.step_func(function(assertion) {
                        assert_equals(assertion.actual, assertion.expected, assertion.message);
                    }));

                    win.close();
                    test.done();
                }
            }));

            // Delete things which can't be cloned and posted over to the new window
            delete config.video;
            delete config.messagehandler;

            // Post the config and session id to the new window when it is ready
            win.onload = function() {
                win.postMessage({config: config, sessionId: _sessionId}, '*');
            }
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            return _video.setMediaKeys( mediaKeys );
        }).then(function() {
            _mediaKeySession = _mediaKeys.createSession('persistent-license');
            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);
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
