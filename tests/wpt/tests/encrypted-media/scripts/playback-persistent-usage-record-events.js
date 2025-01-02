function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', persistent-usage-record, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', playback, check events';

    var configuration = {   initDataTypes: [config.initDataType ],
                            audioCapabilities: [{contentType: config.audioType}],
                            videoCapabilities: [{contentType: config.videoType}],
                            sessionTypes: ['persistent-usage-record']};


    async_test(function(test) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _sessionId,
            _timeupdateEvent = false,
            _events = [ ];

        function recordEventFunc(eventType) {
            return function() { _events.push(eventType); };
        }

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            if (event.messageType !== 'individualization-request') {
                _events.push(event.messageType);
            }

            config.messagehandler(event.messageType, event.message).then(function(response) {
                _events.push(event.messageType + '-response');
                return _mediaKeySession.update(response);
            }).then(test.step_func(function() {
                _events.push('update-resolved');
                if (event.messageType === 'license-release') {
                    checkEventSequence( _events,
                                    ['encrypted','generaterequest-done',
                                        ['license-request', 'license-request-response', 'update-resolved'], // potentially repeating
                                        'keystatuseschange',
                                        'playing',
                                        'remove-resolved',
                                        'keystatuseschange',
                                        'license-release',
                                        'license-release-response',
                                        'closed-attribute-resolved',
                                        'update-resolved' ]);
                    test.done();
                }

                if ( event.messageType === 'license-request' ) {
                    _video.setMediaKeys(_mediaKeys);
                }
            })).catch(onFailure);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            _events.push(event.type);
            _mediaKeySession.generateRequest(   config.initDataType || event.initDataType,
                                                config.initData || event.initData ).then( function() {
                _events.push( 'generaterequest-done' );
                _sessionId = _mediaKeySession.sessionId;
            }).catch(onFailure);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (config.duration || 1) && !_timeupdateEvent) {
                _timeupdateEvent = true;
                _video.pause();
                _mediaKeySession.remove().then(recordEventFunc('remove-resolved')).catch(onFailure);
            }
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, recordEventFunc('playing'), test);

            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);

            _mediaKeySession = _mediaKeys.createSession( 'persistent-usage-record' );
            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            waitForEventAndRunStep('keystatuseschange', _mediaKeySession, recordEventFunc('keystatuseschange'), test);
            _mediaKeySession.closed.then(recordEventFunc('closed-attribute-resolved'));
            return config.servercertificate ? _mediaKeys.setServerCertificate(config.servercertificate) : true;
        }).then(function( success ) {
            return testmediasource(config);
        }).then(function(source) {
            _video.src = URL.createObjectURL(source);
            return source.done;
        }).then(function(){
            _video.play();
        }).catch(onFailure);
    }, testname);
}
