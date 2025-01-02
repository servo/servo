function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', persistent-usage-record, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', playback, retrieve in new window';

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'persistent-usage-record' ] };


    async_test( function( test ) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _sessionId,
            _isClosing = false;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initDataType || event.initDataType,
                                                config.initData || event.initData ).then( function() {
                _sessionId = _mediaKeySession.sessionId;
            }).catch(onFailure);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array(  event.messageType,['license-request', 'individualization-request']);

            config.messagehandler( event.messageType, event.message ).then(function(response) {
                return _mediaKeySession.update(response);
            }).then(function() {
                return _video.setMediaKeys(_mediaKeys);
            }).catch(onFailure);
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        function onTimeupdate(event) {
            if (!_isClosing && _video.currentTime > (config.duration || 1)) {
                _isClosing = true;
                _video.removeEventListener('timeupdate', onTimeupdate);
                _video.pause();
                _mediaKeySession.closed.then( test.step_func(onClosed));
                _mediaKeySession.close();
            }
        }

        function onClosed(event) {
            _video.src = "";
            _video.setMediaKeys( null );

            var win = window.open(config.windowscript);
            assert_not_equals(win, null, "Popup windows not allowed?");

            window.addEventListener('message', test.step_func(function(event) {
                if (event.data.testResult) {
                    event.data.testResult.forEach(test.step_func(function(assertion) {
                        assert_equals(assertion.actual, assertion.expected, assertion.message);
                    }));

                    win.close();
                    test.done();
                }
            }));

            delete config.video;
            delete config.messagehandler;

            win.onload = function() {
                win.postMessage({ config: config, sessionId: _sessionId }, '*');
            }
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            return _video.setMediaKeys(mediaKeys);
        }).then(function(){
            _mediaKeySession = _mediaKeys.createSession( 'persistent-usage-record' );
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
