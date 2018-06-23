// Copyright © 2018 Chromium authors and World Wide Web Consortium, (Massachusetts Institute of Technology, ERCIM, Keio University, Beihang).

function findSupportedChangeTypeTestTypes(cb)
{
    var CHANGE_TYPE_MEDIA_LIST = [
        {
            type: 'video/webm; codecs="vp8"',
            is_video: true,
            url: 'webm/test-v-128k-320x240-24fps-8kfr.webm'
        },
        {
            type: 'video/webm; codecs="vp9"',
            is_video: true,
            url: 'webm/test-vp9.webm'
        },
        {
            type: 'video/mp4; codecs="avc1.4D4001"',
            is_video: true,
            url: 'mp4/test-v-128k-320x240-24fps-8kfr.mp4'
        },
        {
            type: 'video/webm; codecs="vorbis"',
            is_video: false,
            url: 'webm/test-a-128k-44100Hz-1ch.webm'
        },
        {
            type: 'video/mp4; codecs="mp4a.40.2"',
            is_video: false,
            url: 'mp4/test-a-128k-44100Hz-1ch.mp4'
        },
        {
            type: 'audio/mpeg',
            is_video: false,
            url: 'mp3/sound_5.mp3'
        }
    ];

    var audio_result = [];
    var video_result = [];

    for (var i = 0; i < CHANGE_TYPE_MEDIA_LIST.length; ++i) {
        var media = CHANGE_TYPE_MEDIA_LIST[i];
        if (window.MediaSource && MediaSource.isTypeSupported(media.type)) {
            if (media.is_video === true) {
                video_result.push(media);
            } else {
                audio_result.push(media);
            }
        }
    }


    cb(audio_result, video_result);
}

function appendBuffer(test, sourceBuffer, data)
{
    test.expectEvent(sourceBuffer, "update");
    test.expectEvent(sourceBuffer, "updateend");
    sourceBuffer.appendBuffer(data);
}

function trimBuffered(test, mediaElement, sourceBuffer, minimumPreviousDuration, newDuration)
{
    assert_less_than(newDuration, minimumPreviousDuration);
    assert_less_than(minimumPreviousDuration, mediaElement.duration);
    test.expectEvent(sourceBuffer, "update");
    test.expectEvent(sourceBuffer, "updateend");
    sourceBuffer.remove(newDuration, Infinity);
}

function trimDuration(test, mediaElement, mediaSource, newDuration)
{
    assert_less_than(newDuration, mediaElement.duration);
    test.expectEvent(mediaElement, "durationchange");
    mediaSource.duration = newDuration;
}

function runChangeTypeTest(test, mediaElement, mediaSource, typeA, dataA, typeB, dataB)
{
    var sourceBuffer = mediaSource.addSourceBuffer(typeA);

    appendBuffer(test, sourceBuffer, dataA);

    // changeType A->B and append B starting at 0.5 seconds.
    test.waitForExpectedEvents(function()
    {
        sourceBuffer.changeType(typeB);
        sourceBuffer.timestampOffset = 0.5;
        appendBuffer(test, sourceBuffer, dataB);
    });

    // changeType B->B and append B starting at 1.0 seconds.
    test.waitForExpectedEvents(function()
    {
        sourceBuffer.changeType(typeB);
        sourceBuffer.timestampOffset = 1.0;
        appendBuffer(test, sourceBuffer, dataB);
    });

    // changeType B->A and append A starting at 1.5 seconds.
    test.waitForExpectedEvents(function()
    {
        sourceBuffer.changeType(typeA);
        sourceBuffer.timestampOffset = 1.5;
        appendBuffer(test, sourceBuffer, dataA);
    });

    // changeTypoe A->A and append A starting at 1.3 seconds.
    test.waitForExpectedEvents(function()
    {
        sourceBuffer.changeType(typeA);
        sourceBuffer.timestampOffset = 1.3;
        appendBuffer(test, sourceBuffer, dataA);
    });

    // Trim duration to 2 seconds, then play through to end.
    test.waitForExpectedEvents(function()
    {
        trimBuffered(test, mediaElement, sourceBuffer, 2.1, 2);
    });

    test.waitForExpectedEvents(function()
    {
        trimDuration(test, mediaElement, mediaSource, 2);
    });

    test.waitForExpectedEvents(function()
    {
        assert_equals(mediaElement.currentTime, 0);
        test.expectEvent(mediaSource, "sourceended");
        test.expectEvent(mediaElement, "play");
        test.expectEvent(mediaElement, "ended");
        mediaSource.endOfStream();
        mediaElement.play();
    });

    test.waitForExpectedEvents(function() {
        test.done();
    });
}

function mediaSourceChangeTypeTest(metadataA, metadataB, description)
{
    mediasource_test(function(test, mediaElement, mediaSource)
    {
        mediaElement.pause();
        mediaElement.addEventListener('error', test.unreached_func("Unexpected event 'error'"));
        MediaSourceUtil.loadBinaryData(test, metadataA.url, function(dataA) {
            MediaSourceUtil.loadBinaryData(test, metadataB.url, function(dataB) {
                runChangeTypeTest(test, mediaElement, mediaSource, metadataA.type, dataA, metadataB.type, dataB);
            });
        });
    }, description);
}

