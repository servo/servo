function runTest(configEncrypted,configClear,qualifier) {

    var testname = testnamePrefix(qualifier, configEncrypted.keysystem)
                                    + ', temporary, '
                                    + /video\/([^;]*)/.exec(configEncrypted.videoType)[1]
                                    + ', playback, encrypted and clear sources';

    var configuration = {   initDataTypes: [ configEncrypted.initDataType ],
                            audioCapabilities: [ { contentType: configEncrypted.audioType } ],
                            videoCapabilities: [ { contentType: configEncrypted.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    async_test(function(test) {
        var playbackCount = 0,
            _video = configEncrypted.video,
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
            assert_in_array(event.messageType, ['license-request', 'individualization-request']);

            configEncrypted.messagehandler(event.messageType, event.message).then(function(response) {
                return _mediaKeySession.update( response );
            }).catch(onFailure);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(configEncrypted.initData ? configEncrypted.initDataType : event.initDataType,
                                                configEncrypted.initData || event.initData).then(function(){
                return _video.setMediaKeys(_mediaKeys);
            }).catch(onFailure);
        }

        function onPlaying(event)
        {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeUpdate, true);
        }

        function onTimeUpdate(event) {
            if (_video.currentTime < (configEncrypted.duration || 0.5)) {
                return;
            }

            _video.removeEventListener('timeupdate', onTimeUpdate, true);

            resetSrc().then(function(){
                if (playbackCount >= 2) {
                    test.done();
                } else {
                    playbackCount++;
                    startPlayback();
                }
            }).catch(onFailure);
        }

        function resetSrc() {
            _video.pause();
            _video.removeAttribute('src');
            _video.load();
            return _video.setMediaKeys(null);
        }

        function startPlayback() {
            // Alternate between encrypted and unencrypted files.
            if (playbackCount % 2) {
                // Unencrypted files don't require MediaKeys
                testmediasource( configClear ).then(function( source ) {
                    _mediaSource = source;
                    _video.src = URL.createObjectURL(_mediaSource);
                    _video.play();
                }).catch(onFailure);
            } else {
                navigator.requestMediaKeySystemAccess(configEncrypted.keysystem, [ configuration ]).then(function(access) {
                    return access.createMediaKeys();
                }).then(function(mediaKeys) {
                    _mediaKeys = mediaKeys;
                    _mediaKeySession = _mediaKeys.createSession( 'temporary' );
                }).then(function() {
                    return testmediasource(configEncrypted);
                }).then(function(source) {
                    _mediaSource = source;
                    _video.src = URL.createObjectURL(_mediaSource);
                    return source.done;
                }).then(function(){
                    _video.play();
                }).catch(onFailure);
            }
        }

        waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
        waitForEventAndRunStep('playing', _video, onPlaying, test);
        startPlayback();
    }, testname);
}
