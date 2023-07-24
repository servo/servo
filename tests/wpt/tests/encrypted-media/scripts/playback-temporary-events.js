function runTest(config,qualifier) {

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', temporary, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', playback, check events';

    var configuration = {   initDataTypes: [ config.initDataType ],
                            audioCapabilities: [ { contentType: config.audioType } ],
                            videoCapabilities: [ { contentType: config.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    async_test(function(test) {
        var _video = config.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _timeupdateEvent = false,
            _events = [ ];

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');

            assert_in_array(  event.messageType, ['license-request', 'individualization-request']);

            if (event.messageType !== 'individualization-request') {
                _events.push(event.messageType);
            }

            config.messagehandler(event.messageType, event.message).then(function(response) {
                _events.push('license-request-response');
                waitForEventAndRunStep('keystatuseschange', _mediaKeySession, onKeyStatusesChange, test);
                return _mediaKeySession.update( response );
            }).then(function() {
                _events.push('update-resolved');
            }).catch(onFailure);
        }

        function onKeyStatusesChange(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.Event);
            assert_equals(event.type, 'keystatuseschange');
            var hasKeys = false, pendingKeys = false;
            _mediaKeySession.keyStatuses.forEach(function(value, keyid) {
                assert_in_array(value, ['status-pending', 'usable']);
                hasKeys = true;
                pendingKeys = pendingKeys || (value === 'status-pending');
            });

            if (!hasKeys) {
                _events.push('emptykeyslist');
            } else if (!pendingKeys ) {
                _events.push('allkeysusable');
                _video.setMediaKeys(_mediaKeys).catch(onFailure);
            } else {
                assert_unreached('unexpected ' + event.type + ' event');
            }
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(config.initData ? config.initDataType : event.initDataType,
                                                config.initData || event.initData ).then(function() {
                _events.push('generaterequest');
            }).catch(onFailure);
        }

        function onClosed(event) {
            _events.push('closed-attribute-resolved');
            setTimeout(test.step_func(function() {
                checkEventSequence( _events,
                                    ['generaterequest',
                                        ['license-request', 'license-request-response', 'update-resolved'], // potentially repeating
                                        'allkeysusable',
                                        'playing',
                                        'closed-attribute-resolved',
                                        'close-promise-resolved',
                                        'emptykeyslist']);
                test.done();
            } ), 0);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (config.duration || 1) && !_timeupdateEvent) {
                _timeupdateEvent = true;
                _video.pause();

                _mediaKeySession.closed.then(test.step_func(onClosed));
                _mediaKeySession.close().then(function() {
                    _events.push('close-promise-resolved');
                }).catch(onFailure);
            }
        }

        function onPlaying(event) {
            _events.push('playing');

            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            _mediaKeySession = _mediaKeys.createSession('temporary');

            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);
        }).then(function() {
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
