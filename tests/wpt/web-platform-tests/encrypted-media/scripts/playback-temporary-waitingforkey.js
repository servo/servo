function runTest(config,qualifier) {

    // config.initData contains a list of keys. We expect those to be needed in order and get
    // one waitingforkey event for each one.

    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', successful playback, temporary, '
                                    + /video\/([^;]*)/.exec(config.videoType)[1]
                                    + ', waitingforkey event, '
                                    + config.initData.length + ' key' + (config.initData.length > 1 ? 's' : '');

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
            config.messagehandler( event.messageType, event.message ).then( function( response ) {
                return event.target.update( response );
            }).catch(onFailure);
        }

        function onWaitingForKey(event) {
            // Expect one waitingforkey event for each initData we were given
            assert_less_than(_mediaKeySessions.length, config.initData.length);
            var mediaKeySession = _mediaKeys.createSession( 'temporary' );
            waitForEventAndRunStep('message', mediaKeySession, onMessage, test);
            _mediaKeySessions.push(mediaKeySession);
            mediaKeySession.generateRequest(config.initDataType, config.initData[_mediaKeySessions.length - 1]).catch(onFailure);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (config.duration || 1)) {
                assert_equals(_mediaKeySessions.length, config.initData.length);
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
        }).then(function(){
            waitForEventAndRunStep('waitingforkey', _video, onWaitingForKey, test);

            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
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
