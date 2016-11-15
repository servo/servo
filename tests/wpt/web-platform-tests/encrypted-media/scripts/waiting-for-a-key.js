function runTest(config)
{
    // For debugging timeouts, keep track of the number of the
    // various events received.
    var debugEncryptedEventFired = false;
    var debugWaitingForKeyEventFired = false;
    var debugTimeUpdateEventCount = 0;
    var debugMessage = '';

    // Set global option explicit_timeout to true and control
    // the timeout in the promise test below.
    setup({
        explicit_timeout: true
    });

    promise_test(function (test) {
        var video = config.video;
        var keysystem = config.keysystem;
        var configuration = {
            initDataTypes: [config.initDataType],
            audioCapabilities: [{
                contentType: config.audioType
            }],
            videoCapabilities: [{
                contentType: config.videoType
            }],
            sessionTypes: ['temporary']
        };
        var initData;
        var initDataType;
        var mediaKeySession;
        // As this code doesn't wait for the 'message' event for clearkey to avoid
        // race conditions with 'waitingforkey', specify the key ID and
        // key used by the encrypted content.
        var keyId = new Uint8Array(config.content.keys[0].kid);
        var rawKey = new Uint8Array(config.content.keys[0].key);
        // Use the message handler for non clearkey drm
        var handler = config.messageHandler || null;

        // Override timeout() to use custom message instead of default
        // message "Test timed out"
        test.timeout = function () {
            var message = 'timeout. message = ' + debugMessage
              + ', encrypted: ' + debugEncryptedEventFired
              + ', waitingforkey: ' + debugWaitingForKeyEventFired
              + ', timeupdate count: ' + debugTimeUpdateEventCount;

            this.timeout_id = null;
            this.set_status(this.TIMEOUT, message);
            this.phase = this.phases.HAS_RESULT;
            this.done();
        };

        return navigator.requestMediaKeySystemAccess(keysystem, [configuration]).then(function (access) {
            debugMessage = 'createMediaKeys()';
            return access.createMediaKeys();
        }).then(function (mediaKeys) {
              debugMessage = 'setMediaKeys()';
              return video.setMediaKeys(mediaKeys);
        }).then(function () {
            return testmediasource(config);
        }).then(function (source) {
            debugMessage = 'wait_for_encrypted_event()';
            mediaSource = source;
            video.src = URL.createObjectURL(mediaSource);
            video.play();
            return wait_for_encrypted_event(video);
        }).then(function (e) {
            // Received the 'encrypted' event(s), so keep a copy of
            // the initdata for use when creating the session later.
            initDataType = config.initData ? config.initDataType : e.initDataType;
            initData = config.initData || e.initData;
            // Wait until the video indicates that it needs a key to
            // continue.
            debugMessage = 'wait_for_waitingforkey_event()';
            return wait_for_waitingforkey_event(video);
        }).then(function () {
            // Make sure the video is NOT paused and not progressing
            // before a key is provided. This requires the video
            // to NOT have a clear lead.
            assert_false(video.paused);
            assert_less_than(video.currentTime, 0.2);
            // Create a session.
            mediaKeySession = video.mediaKeys.createSession('temporary');
            debugMessage = 'generateRequest()';
            return mediaKeySession.generateRequest(initDataType, initData);
        }).then(function () {
            // generateRequest() will cause a 'message' event to
            // occur specifying the keyId that is needed
            // Add the key needed to decrypt.
            return wait_for_message_event(mediaKeySession, handler);
        }).then(function () {
            // Video should start playing now that it can decrypt the
            // streams, so wait until a little bit of the video has
            // played.
            debugMessage = 'wait_for_timeupdate_event()';
            return wait_for_timeupdate_event(video);
        }).catch(function (error) {
            assert_unreached('Error: ' + error.name);
        });

        // Typical test duration is 6 seconds on release builds
        // (12 seconds on debug).
    }, 'Waiting for a key.', {timeout: 20000});

    // Wait for an 'encrypted' event
    function wait_for_encrypted_event(video)
    {
        return new Promise(function (resolve) {
            video.addEventListener('encrypted', function listener(e) {
                assert_equals(e.target, video);
                assert_true(e instanceof window.MediaEncryptedEvent);
                assert_equals(e.type, 'encrypted');
                debugEncryptedEventFired = true;
                video.removeEventListener('encrypted', listener);
                resolve(e);
            });
        });
    };

    // Wait for a 'waitingforkey' event. Promise resolved when the
    // event is received.
    function wait_for_waitingforkey_event(video)
    {
        return new Promise(function (resolve) {
            video.addEventListener('waitingforkey', function listener(e) {
                assert_equals(e.target, video);
                assert_equals(e.type, 'waitingforkey');
                debugWaitingForKeyEventFired = true;
                video.removeEventListener('waitingforkey', listener);
                resolve(e);
            });
        });
    };

    // Wait for a 'timeupdate' event. Promise resolved if |video| has
    // played for more than 0.2 seconds.
    function wait_for_timeupdate_event(video)
    {
        return new Promise(function (resolve) {
            video.addEventListener('timeupdate', function listener(e) {
                assert_equals(e.target, video);
                ++debugTimeUpdateEventCount;
                if (video.currentTime < 0.2)
                    return;
                video.removeEventListener('timeupdate', listener);
                resolve(e);
            });
        });
    };

    // We need to wait for the message even if for non clearkey DRMs.
    function wait_for_message_event(mediaKeySession, handler)
    {
        return new Promise(function (resolve, reject) {
            mediaKeySession.addEventListener('message', function listener(e) {
                assert_equals(e.target, mediaKeySession);
                assert_equals(e.type, 'message');
                video.removeEventListener('message', listener);
                return handler(e.messageType, e.message).then(function (response) {
                    return e.target.update(response)
                }).then(resolve, reject);
            });
        });
    }
}
