function runTest(config)
{
    async_test(function(test) {
        var mediaKeys;
        var mediaSource;
        var encryptedEventIndex = 0;
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

        assert_not_equals(video, null);

        var onEncrypted = function(event) {
            ++encryptedEventIndex;
            assert_equals(video.mediaKeys, mediaKeys);

            // This event is fired once for the audio stream and once
            // for the video stream each time .src is set.
            if (encryptedEventIndex === 2) {
                // Finished first video; Create new media source and wait for two more encrypted events
                return testmediasource(config).then(function (source) {
                    video.src = URL.createObjectURL(source);
                }).catch(function (error) {
                    forceTestFailureFromPromise(test, error)
                });
            } else if (encryptedEventIndex === 4) {
                // Finished second video.
                test.done();
            }
        };

        // Create a MediaKeys object and assign it to video.
        navigator.requestMediaKeySystemAccess(keysystem, [configuration]).then(test.step_func(function (access) {
            assert_equals(access.keySystem, keysystem);
            return access.createMediaKeys();
        })).then(test.step_func(function (result) {
            mediaKeys = result;
            assert_not_equals(mediaKeys, null);
            return video.setMediaKeys(mediaKeys);
        })).then(test.step_func(function () {
            assert_equals(video.mediaKeys, mediaKeys);
            return testmediasource(config);
        })).then(function (source) {
            waitForEventAndRunStep('encrypted', video, onEncrypted, test);
            mediaSource = source;
            video.src = URL.createObjectURL(mediaSource);
        }).catch(function (error) {
            forceTestFailureFromPromise(test, error);
        });

    }, 'Reset src after setMediaKeys().');
}
