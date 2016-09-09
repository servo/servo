// Copyright © 2016 Chromium authors and World Wide Web Consortium, (Massachusetts Institute of Technology, ERCIM, Keio University, Beihang).

(function(window) {
    var SEGMENT_INFO_LIST = [
        {
            url: 'mp4/test.mp4',
            type: 'video/mp4; codecs="mp4a.40.2,avc1.4d400d"',
            duration: 6.0756,
            init: { offset: 0, size: 1197 },
            media: [
                { offset: 1241, size: 17845, timev: 0.033200, timea: 0, endtimev: 0.531200, endtimea: 0.510839 },
                { offset: 19130, size: 5551, timev: 0.464800, timea: 0.510839, endtimev: 0.796800, endtimea: 0.812698 },
                { offset: 24725, size: 10944, timev: 0.796800, timea: 0.812698, endtimev: 0.929600, endtimea: 0.905578 },
                { offset: 35713, size: 7131, timev: 0.863200, timea: 0.905578, endtimev: 1.195200, endtimea: 1.184217 },
                { offset: 42888, size: 2513, timev: 1.128800, timea: 1.184217, endtimev: 1.328000, endtimea: 1.300317 },
                { offset: 45457, size: 3022, timev: 1.261600, timea: 1.300317, endtimev: 1.460800, endtimea: 1.509297 },
                { offset: 48479, size: 815, timev: 1.494000, timea: 1.509297, endtimev: 1.527200, endtimea: 1.532517 },
                { offset: 49338, size: 2818, timev: 1.460800, timea: 1.532517, endtimev: 1.626800, endtimea: 1.648616 },
                { offset: 52200, size: 11581, timev: 1.626800, timea: 1.648616, endtimev: 1.792800, endtimea: 1.764716 },
                { offset: 63825, size: 3003, timev: 1.726400, timea: 1.764716, endtimev: 1.925600, endtimea: 1.973696 },
                { offset: 66872, size: 6390, timev: 1.925600, timea: 1.973696, endtimev: 2.191200, endtimea: 2.159455 },
                { offset: 73306, size: 3740, timev: 2.124800, timea: 2.159455, endtimev: 2.390400, endtimea: 2.368435 },
                { offset: 77102, size: 11779, timev: 2.324000, timea: 2.368435, endtimev: 2.523200, endtimea: 2.577414 },
                { offset: 88881, size: 851, timev: 2.556400, timea: 2.577414, endtimev: 2.589600, endtimea: 2.600634 },
                { offset: 89776, size: 4236, timev: 2.523200, timea: 2.600634, endtimev: 2.788800, endtimea: 2.832834 },
                { offset: 94056, size: 9538, timev: 2.788800, timea: 2.832834, endtimev: 3.187200, endtimea: 3.204353 },
                { offset: 103638, size: 13295, timev: 3.187200, timea: 3.204353, endtimev: 3.452800, endtimea: 3.436553 },
                { offset: 116977, size: 309, timev: 3.386400, timea: 3.436553, endtimev: 3.419600, endtimea: 3.506213 },
                { offset: 117330, size: 5806, timev: 3.452800, timea: 3.506213, endtimev: 3.784800, endtimea: 3.831292 },
                { offset: 123180, size: 4392, timev: 3.784800, timea: 3.831292, endtimev: 4.017200, endtimea: 4.040272 },
                { offset: 127616, size: 15408, timev: 4.017200, timea: 4.040272, endtimev: 4.249600, endtimea: 4.295691 },
                { offset: 143068, size: 9899, timev: 4.249600, timea: 4.295691, endtimev: 4.814000, endtimea: 4.829750 },
                { offset: 153011, size: 11562, timev: 4.814000, timea: 4.829750, endtimev: 4.980000, endtimea: 5.015510 },
                { offset: 164617, size: 7398, timev: 4.980000, timea: 5.015510, endtimev: 5.245600, endtimea: 5.294149 },
                { offset: 172059, size: 5698, timev: 5.245600, timea: 5.294149, endtimev: 5.577600, endtimea: 5.549569 },
                { offset: 177801, size: 11682, timev: 5.511200, timea: 5.549569, endtimev: 5.710400, endtimea: 5.758548 },
                { offset: 189527, size: 3023, timev: 5.710400, timea: 5.758548, endtimev: 5.909600, endtimea: 5.897868 },
                { offset: 192594, size: 5726, timev: 5.843200, timea: 5.897868, endtimev: 6.075600, endtimea: 6.037188 },
            ]
        },
        {
            url: 'webm/test.webm',
            type: 'video/webm; codecs="vp8, vorbis"',
            duration: 6.042,
            init: { offset: 0, size: 4357 },
            media: [
                {  offset: 4357, size: 11830, timev: 0, timea: 0, endtimev: 0.398000, endtimea: 0.384000 },
                {  offset: 16187, size: 12588, timev: 0.398000, timea: 0.385000, endtimev: 0.798000, endtimea: 0.779000 },
                {  offset: 28775, size: 14588, timev: 0.797000, timea: 0.779000, endtimev: 1.195000, endtimea: 1.174000 },
                {  offset: 43363, size: 13023, timev: 1.195000, timea: 1.174000, endtimev: 1.593000, endtimea: 1.592000 },
                {  offset: 56386, size: 13127, timev: 1.594000, timea: 1.592000, endtimev: 1.992000, endtimea: 1.988000 },
                {  offset: 69513, size: 14456, timev: 1.992000, timea: 1.987000, endtimev: 2.390000, endtimea: 2.381000 },
                {  offset: 83969, size: 13458, timev: 2.390000, timea: 2.381000, endtimev: 2.790000, endtimea: 2.776000 },
                {  offset: 97427, size: 14566, timev: 2.789000, timea: 2.776000, endtimev: 3.187000, endtimea: 3.171000 },
                {  offset: 111993, size: 13201, timev: 3.187000, timea: 3.171000, endtimev: 3.585000, endtimea: 3.565000 },
                {  offset: 125194, size: 14061, timev: 3.586000, timea: 3.566000, endtimev: 3.984000, endtimea: 3.960000 },
                {  offset: 139255, size: 15353, timev: 3.984000, timea: 3.960000, endtimev: 4.382000, endtimea: 4.378000 },
                {  offset: 154608, size: 13618, timev: 4.382000, timea: 4.378000, endtimev: 4.782000, endtimea: 4.773000 },
                {  offset: 168226, size: 15094, timev: 4.781000, timea: 4.773000, endtimev: 5.179000, endtimea: 5.169000 },
                {  offset: 183320, size: 13069, timev: 5.179000, timea: 5.168000, endtimev: 5.577000, endtimea: 5.562000 },
                {  offset: 196389, size: 13788, timev: 5.578000, timea: 5.563000, endtimev: 5.976000, endtimea: 5.957000 },
                {  offset: 210177, size: 9009, timev: 5.976000, timea: 5.957000, endtimev: 6.042000, endtimea: 6.050000 },
            ],
        }
    ];
    EventExpectationsManager = function(test)
    {
        this.test_ = test;
        this.eventTargetList_ = [];
        this.waitCallbacks_ = [];
    };

    EventExpectationsManager.prototype.expectEvent = function(object, eventName, description)
    {
        var eventInfo = { 'target': object, 'type': eventName, 'description': description};
        var expectations = this.getExpectations_(object);
        expectations.push(eventInfo);

        var t = this;
        var waitHandler = this.test_.step_func(this.handleWaitCallback_.bind(this));
        var eventHandler = this.test_.step_func(function(event)
        {
            object.removeEventListener(eventName, eventHandler);
            var expected = expectations[0];
            assert_equals(event.target, expected.target, "Event target match.");
            assert_equals(event.type, expected.type, "Event types match.");
            assert_equals(eventInfo.description, expected.description, "Descriptions match for '" +  event.type + "'.");

            expectations.shift(1);
            if (t.waitCallbacks_.length > 1)
                setTimeout(waitHandler, 0);
            else if (t.waitCallbacks_.length == 1) {
                // Immediately call the callback.
                waitHandler();
            }
        });
        object.addEventListener(eventName, eventHandler);
    };

    EventExpectationsManager.prototype.waitForExpectedEvents = function(callback)
    {
        this.waitCallbacks_.push(callback);
        setTimeout(this.test_.step_func(this.handleWaitCallback_.bind(this)), 0);
    };

    EventExpectationsManager.prototype.expectingEvents = function()
    {
        for (var i = 0; i < this.eventTargetList_.length; ++i) {
            if (this.eventTargetList_[i].expectations.length > 0) {
                return true;
            }
        }
        return false;
    }

    EventExpectationsManager.prototype.handleWaitCallback_ = function()
    {
        if (this.waitCallbacks_.length == 0 || this.expectingEvents())
            return;
        var callback = this.waitCallbacks_.shift(1);
        callback();
    };

    EventExpectationsManager.prototype.getExpectations_ = function(target)
    {
        for (var i = 0; i < this.eventTargetList_.length; ++i) {
            var info = this.eventTargetList_[i];
            if (info.target == target) {
                return info.expectations;
            }
        }
        var expectations = [];
        this.eventTargetList_.push({ 'target': target, 'expectations': expectations });
        return expectations;
    };

    function loadData_(test, url, callback, isBinary)
    {
        var request = new XMLHttpRequest();
        request.open("GET", url, true);
        if (isBinary) {
            request.responseType = 'arraybuffer';
        }
        request.onload = test.step_func(function(event)
        {
            if (request.status != 200) {
                assert_unreached("Unexpected status code : " + request.status);
                return;
            }
            var response = request.response;
            if (isBinary) {
                response = new Uint8Array(response);
            }
            callback(response);
        });
        request.onerror = test.step_func(function(event)
        {
            assert_unreached("Unexpected error");
        });
        request.send();
    }

    function openMediaSource_(test, mediaTag, callback)
    {
        var mediaSource = new MediaSource();
        var mediaSourceURL = URL.createObjectURL(mediaSource);

        var eventHandler = test.step_func(onSourceOpen);
        function onSourceOpen(event)
        {
            mediaSource.removeEventListener('sourceopen', eventHandler);
            URL.revokeObjectURL(mediaSourceURL);
            callback(mediaSource);
        }

        mediaSource.addEventListener('sourceopen', eventHandler);
        mediaTag.src = mediaSourceURL;
    }

    var MediaSourceUtil = {};

    MediaSourceUtil.loadTextData = function(test, url, callback)
    {
        loadData_(test, url, callback, false);
    };

    MediaSourceUtil.loadBinaryData = function(test, url, callback)
    {
        loadData_(test, url, callback, true);
    };

    MediaSourceUtil.fetchManifestAndData = function(test, manifestFilename, callback)
    {
        var baseURL = '';
        var manifestURL = baseURL + manifestFilename;
        MediaSourceUtil.loadTextData(test, manifestURL, function(manifestText)
        {
            var manifest = JSON.parse(manifestText);

            assert_true(MediaSource.isTypeSupported(manifest.type), manifest.type + " is supported.");

            var mediaURL = baseURL + manifest.url;
            MediaSourceUtil.loadBinaryData(test, mediaURL, function(mediaData)
            {
                callback(manifest.type, mediaData);
            });
        });
    };

    MediaSourceUtil.extractSegmentData = function(mediaData, info)
    {
        var start = info.offset;
        var end = start + info.size;
        return mediaData.subarray(start, end);
    }

    MediaSourceUtil.getMediaDataForPlaybackTime = function(mediaData, segmentInfo, playbackTimeToAdd)
    {
        assert_less_than_equal(playbackTimeToAdd, segmentInfo.duration);
        var mediaInfo = segmentInfo.media;
        var start = mediaInfo[0].offset;
        var numBytes = 0;
        var segmentIndex = 0;
        while (segmentIndex < mediaInfo.length
               && Math.min(mediaInfo[segmentIndex].timev, mediaInfo[segmentIndex].timea) <= playbackTimeToAdd)
        {
          numBytes += mediaInfo[segmentIndex].size;
          ++segmentIndex;
        }
        return mediaData.subarray(start, numBytes + start);
    }

    function getFirstSupportedType(typeList)
    {
        for (var i = 0; i < typeList.length; ++i) {
            if (window.MediaSource && MediaSource.isTypeSupported(typeList[i]))
                return typeList[i];
        }
        return "";
    }

    function getSegmentInfo()
    {
        for (var i = 0; i < SEGMENT_INFO_LIST.length; ++i) {
            var segmentInfo = SEGMENT_INFO_LIST[i];
            if (window.MediaSource && MediaSource.isTypeSupported(segmentInfo.type)) {
                return segmentInfo;
            }
        }
        return null;
    }

    var audioOnlyTypes = ['audio/mp4;codecs="mp4a.40.2"', 'audio/webm;codecs="vorbis"'];
    var videoOnlyTypes = ['video/mp4;codecs="avc1.4D4001"', 'video/webm;codecs="vp8"'];
    var audioVideoTypes = ['video/mp4;codecs="avc1.4D4001,mp4a.40.2"', 'video/webm;codecs="vp8,vorbis"'];
    MediaSourceUtil.AUDIO_ONLY_TYPE = getFirstSupportedType(audioOnlyTypes);
    MediaSourceUtil.VIDEO_ONLY_TYPE = getFirstSupportedType(videoOnlyTypes);
    MediaSourceUtil.AUDIO_VIDEO_TYPE = getFirstSupportedType(audioVideoTypes);
    MediaSourceUtil.SEGMENT_INFO = getSegmentInfo();

    MediaSourceUtil.getSubType = function(mimetype) {
        var slashIndex = mimetype.indexOf("/");
        var semicolonIndex = mimetype.indexOf(";");
        if (slashIndex <= 0) {
            assert_unreached("Invalid mimetype '" + mimetype + "'");
            return;
        }

        var start = slashIndex + 1;
        if (semicolonIndex >= 0) {
            if (semicolonIndex <= start) {
                assert_unreached("Invalid mimetype '" + mimetype + "'");
                return;
            }

            return mimetype.substr(start, semicolonIndex - start)
        }

        return mimetype.substr(start);
    };

    MediaSourceUtil.append = function(test, sourceBuffer, data, callback)
    {
        function onUpdate() {
            sourceBuffer.removeEventListener("update", onUpdate);
            callback();
        }
        sourceBuffer.addEventListener("update", onUpdate);

        sourceBuffer.addEventListener('error', test.unreached_func("Unexpected event 'error'"));

        sourceBuffer.appendBuffer(data);
    };

    MediaSourceUtil.appendUntilEventFires = function(test, mediaElement, eventName, sourceBuffer, mediaData, segmentInfo, startingIndex)
    {
        var eventFired = false;
        function onEvent() {
            mediaElement.removeEventListener(eventName, onEvent);
            eventFired = true;
        }
        mediaElement.addEventListener(eventName, onEvent);

        var i = startingIndex;
        var onAppendDone = function() {
            if (eventFired || (i >= (segmentInfo.media.length - 1)))
                return;

            i++;
            if (i < segmentInfo.media.length)
            {
                MediaSourceUtil.append(test, sourceBuffer, MediaSourceUtil.extractSegmentData(mediaData, segmentInfo.media[i]), onAppendDone);
            }
        };
        MediaSourceUtil.append(test, sourceBuffer, MediaSourceUtil.extractSegmentData(mediaData, segmentInfo.media[i]), onAppendDone);

    };

    function addExtraTestMethods(test)
    {
        test.eventExpectations_ = new EventExpectationsManager(test);
        test.expectEvent = function(object, eventName, description)
        {
            test.eventExpectations_.expectEvent(object, eventName, description);
        };

        test.waitForExpectedEvents = function(callback)
        {
            test.eventExpectations_.waitForExpectedEvents(callback);
        };

        test.waitForCurrentTimeChange = function(mediaElement, callback)
        {
            var initialTime = mediaElement.currentTime;

            var onTimeUpdate = test.step_func(function()
            {
                if (mediaElement.currentTime != initialTime) {
                    mediaElement.removeEventListener('timeupdate', onTimeUpdate);
                    callback();
                }
            });

            mediaElement.addEventListener('timeupdate', onTimeUpdate);
        }

        var oldTestDone = test.done.bind(test);
        test.done = function()
        {
            if (test.status == test.PASS) {
                assert_false(test.eventExpectations_.expectingEvents(), "No pending event expectations.");
            }
            oldTestDone();
        };
    };

    window['MediaSourceUtil'] = MediaSourceUtil;
    window['media_test'] = function(testFunction, description, options)
    {
        options = options || {};
        return async_test(function(test)
        {
            addExtraTestMethods(test);
            testFunction(test);
        }, description, options);
    };
    window['mediasource_test'] = function(testFunction, description, options)
    {
        return media_test(function(test)
        {
            var mediaTag = document.createElement("video");
            if (!document.body) {
                document.body = document.createElement("body");
            }
            document.body.appendChild(mediaTag);

            test.removeMediaElement_ = true;
            test.add_cleanup(function()
            {
                if (test.removeMediaElement_) {
                    document.body.removeChild(mediaTag);
                    test.removeMediaElement_ = false;
                }
            });

            openMediaSource_(test, mediaTag, function(mediaSource)
            {
                testFunction(test, mediaTag, mediaSource);
            });
        }, description, options);
    };

    window['mediasource_testafterdataloaded'] = function(testFunction, description, options)
    {
        mediasource_test(function(test, mediaElement, mediaSource)
        {
            var segmentInfo = MediaSourceUtil.SEGMENT_INFO;

            if (!segmentInfo) {
                assert_unreached("No segment info compatible with this MediaSource implementation.");
                return;
            }

            mediaElement.addEventListener('error', test.unreached_func("Unexpected event 'error'"));

            var sourceBuffer = mediaSource.addSourceBuffer(segmentInfo.type);
            MediaSourceUtil.loadBinaryData(test, segmentInfo.url, function(mediaData)
            {
                testFunction(test, mediaElement, mediaSource, segmentInfo, sourceBuffer, mediaData);
            });
        }, description, options);
    }

    function timeRangesToString(ranges)
    {
        var s = "{";
        for (var i = 0; i < ranges.length; ++i) {
            s += " [" + ranges.start(i).toFixed(3) + ", " + ranges.end(i).toFixed(3) + ")";
        }
        return s + " }";
    }

    window['assertBufferedEquals'] = function(obj, expected, description)
    {
        var actual = timeRangesToString(obj.buffered);
        assert_equals(actual, expected, description);
    };

    window['assertSeekableEquals'] = function(obj, expected, description)
    {
        var actual = timeRangesToString(obj.seekable);
        assert_equals(actual, expected, description);
    };

})(window);
