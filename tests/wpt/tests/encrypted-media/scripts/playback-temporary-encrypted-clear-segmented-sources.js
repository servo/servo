function runTest(configEncrypted,configClear,qualifier) {

    var testname = testnamePrefix(qualifier, configEncrypted.keysystem)
                                    + ', temporary, '
                                    + /video\/([^;]*)/.exec(configEncrypted.videoType)[1]
                                    + ', playback, encrypted and clear sources in separate segments';

    var configuration = {   initDataTypes: [ configEncrypted.initDataType ],
                            audioCapabilities: [ { contentType: configEncrypted.audioType } ],
                            videoCapabilities: [ { contentType: configEncrypted.videoType } ],
                            sessionTypes: [ 'temporary' ] };

    async_test(function(test) {
        var didAppendEncrypted = false,
            _video = configEncrypted.video,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource,
            _sourceBuffer;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onVideoError(event) {
            var message = (_video.error || {}).message || 'Got unknown error from <video>';
            forceTestFailureFromPromise(test, new Error(message));
        }

        function onMessage(event) {
            assert_equals(event.target, _mediaKeySession);
            assert_true(event instanceof window.MediaKeyMessageEvent);
            assert_equals(event.type, 'message');
            assert_in_array(event.messageType, ['license-request', 'individualization-request']);

            configEncrypted.messagehandler(event.messageType, event.message).then(function(response) {
                return _mediaKeySession.update(response);
            }).catch(onFailure);
        }

        function onEncrypted(event) {
            assert_equals(event.target, _video);
            assert_true(event instanceof window.MediaEncryptedEvent);
            assert_equals(event.type, 'encrypted');

            var initDataType = configEncrypted.initDataType || event.initDataType;
            var initData = configEncrypted.initData || event.initData;

            _mediaKeySession = _mediaKeys.createSession('temporary');
            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(initDataType, initData).catch(onFailure);
        }

        function onPlaying(event) {
            // Not using waitForEventAndRunStep() to avoid too many
            // EVENT(onTimeUpdate) logs.
            _video.addEventListener('timeupdate', onTimeupdate, true);
        }

        function onTimeupdate(event) {
            if (_video.currentTime > (configEncrypted.duration || 1) + (configClear.duration || 1)) {
                _video.pause();
                test.done();
            }
            if (_video.currentTime > 1 && !didAppendEncrypted) {
                didAppendEncrypted = true;
                _sourceBuffer.timestampOffset = configClear.duration;
                fetchAndAppend(configEncrypted.videoPath).then(function() {
                  _mediaSource.endOfStream();
                }).catch(onFailure);
            }
        }

        function fetchAndAppend(path) {
            return fetch(path).then(function(response) {
                if (!response.ok) throw new Error('Resource fetch failed');
                return response.arrayBuffer();
            }).then(function(data) {
                return new Promise(function(resolve, reject) {
                    _sourceBuffer.appendBuffer(data);
                    _sourceBuffer.addEventListener('updateend', resolve);
                    _sourceBuffer.addEventListener('error', reject);
                });
            });
        }

        _video.addEventListener('error', onVideoError);
        navigator.requestMediaKeySystemAccess(configEncrypted.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            _mediaKeys = mediaKeys;
            return _video.setMediaKeys(_mediaKeys);
        }).then(function(){
            waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
            waitForEventAndRunStep('playing', _video, onPlaying, test);

            return new Promise(function(resolve, reject) {
                _mediaSource = new MediaSource();
                _mediaSource.addEventListener('sourceopen', resolve);
                _video.src = URL.createObjectURL(_mediaSource);
            });
        }).then(function() {
            _sourceBuffer = _mediaSource.addSourceBuffer(configEncrypted.videoType);
            return fetchAndAppend(configClear.videoPath);
        }).then(function() {
            _video.play();
        }).catch(onFailure);
    }, testname);
}
