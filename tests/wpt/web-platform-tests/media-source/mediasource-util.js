(function(window) {
    var SEGMENT_INFO_LIST = [
        {
            url: 'mp4/test.mp4',
            type: 'video/mp4; codecs="mp4a.40.2,avc1.4d400d"',
            duration: 6.0756,
            init: { offset: 0, size: 1197 },
            media: [
                { offset: 1241, size: 17845, timecode: 0.000000 },
                { offset: 19130, size: 5551, timecode: 0.464800 },
                { offset: 24725, size: 10944, timecode: 0.763600 },
                { offset: 35713, size: 7131, timecode: 0.863200 },
                { offset: 42888, size: 2513, timecode: 1.128800 },
                { offset: 45457, size: 3022, timecode: 1.261600 },
                { offset: 48479, size: 815, timecode: 1.427600 },
                { offset: 49338, size: 2818, timecode: 1.460800 },
                { offset: 52200, size: 11581, timecode: 1.593600 },
                { offset: 63825, size: 3003, timecode: 1.726400 },
                { offset: 66872, size: 6390, timecode: 1.892400 },
                { offset: 73306, size: 3740, timecode: 2.124800 },
                { offset: 77102, size: 11779, timecode: 2.324000 },
                { offset: 88881, size: 851, timecode: 2.490000 },
                { offset: 89776, size: 4236, timecode: 2.523200 },
                { offset: 94056, size: 9538, timecode: 2.755600 },
                { offset: 103638, size: 13295, timecode: 3.154000 },
                { offset: 116977, size: 309, timecode: 3.386400 },
                { offset: 117330, size: 5806, timecode: 3.419600 },
                { offset: 123180, size: 4392, timecode: 3.751600 },
                { offset: 127616, size: 15408, timecode: 3.984000 },
                { offset: 143068, size: 9899, timecode: 4.216400 },
                { offset: 153011, size: 11562, timecode: 4.780800 },
                { offset: 164617, size: 7398, timecode: 4.946800 },
                { offset: 172059, size: 5698, timecode: 5.212400 },
                { offset: 177801, size: 11682, timecode: 5.511200 },
                { offset: 189527, size: 3023, timecode: 5.677200 },
                { offset: 192594, size: 5726, timecode: 5.843200 },
            ]
        },
        {
            url: 'webm/test.webm',
            type: 'video/webm; codecs="vp8, vorbis"',
            duration: 6.042,
            init: { offset: 0, size: 4357 },
            media: [
                {  offset: 4357, size: 11830, timecode: 0 },
                {  offset: 16187, size: 12588, timecode: 0.385 },
                {  offset: 28775, size: 14588, timecode: 0.779 },
                {  offset: 43363, size: 13023, timecode: 1.174 },
                {  offset: 56386, size: 13127, timecode: 1.592 },
                {  offset: 69513, size: 14456, timecode: 1.987 },
                {  offset: 83969, size: 13458, timecode: 2.381 },
                {  offset: 97427, size: 14566, timecode: 2.776 },
                {  offset: 111993, size: 13201, timecode: 3.171 },
                {  offset: 125194, size: 14061, timecode: 3.566 },
                {  offset: 139255, size: 15353, timecode: 3.96 },
                {  offset: 154608, size: 13618, timecode: 4.378 },
                {  offset: 168226, size: 15094, timecode: 4.773 },
                {  offset: 183320, size: 13069, timecode: 5.168 },
                {  offset: 196389, size: 13788, timecode: 5.563 },
                {  offset: 210177, size: 9009, timecode: 5.957 },
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
            if (t.waitCallbacks_.length > 0)
                setTimeout(waitHandler, 0);
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
        while (segmentIndex < mediaInfo.length && mediaInfo[segmentIndex].timecode <= playbackTimeToAdd)
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
            if (eventFired)
                return;

            i++;
            MediaSourceUtil.append(test, sourceBuffer, MediaSourceUtil.extractSegmentData(mediaData, segmentInfo.media[i]), onAppendDone);
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
