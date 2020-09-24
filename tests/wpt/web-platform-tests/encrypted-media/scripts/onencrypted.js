function runTest(config) {
    var expectedInitData = [];
    expectedInitData.push(stringToUint8Array(atob(config.keys[0].initData)));
    expectedInitData.push(stringToUint8Array(atob(config.keys[1].initData)));

    // Will get 2 identical events, one for audio, one for video.
    var expectedEvents = 2;
    var currentData;

    async_test(function (test) {
        var video = config.video,
            mediaSource,
            onEncrypted = function (event) {
                currentData = new Uint8Array(event.initData);
                assert_equals(event.target, config.video);
                assert_true(event instanceof window.MediaEncryptedEvent);
                assert_equals(event.type, 'encrypted');
                assert_equals(event.initDataType, 'cenc');
                // At this point we do not know if the event is related to audio or video. So check for both expected init data
                assert_true(checkInitData(currentData, expectedInitData[0]) || checkInitData(currentData, expectedInitData[1]));

                if (--expectedEvents === 0) {
                    test.done();
                }
            };

        waitForEventAndRunStep('encrypted', video, onEncrypted, test);
        testmediasource(config).then(function (source) {
            mediaSource = source;
            config.video.src = URL.createObjectURL(mediaSource);
            return source.done;
        }).then(function(){
            video.play();
        });
    }, 'encrypted fired on encrypted media file.');
}

function checkInitData(data, expectedData) {
    if (data.length !== expectedData.length) {
        return false;
    }
    for (var i = 0; i < data.length; i++) {
        if (data[i] !== expectedData[i]) {
            return false;
        }
    }
    return true;
}
