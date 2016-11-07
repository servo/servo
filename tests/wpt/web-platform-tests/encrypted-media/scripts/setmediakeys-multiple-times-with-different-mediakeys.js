function runTest(config, qualifier) {
    var testname = testnamePrefix( qualifier, config.keysystem )
                         + ', setmediakeys multiple times with different mediakeys';

    var configuration = getSimpleConfigurationForContent( config.content );

    async_test (function (test) {
        var _video = config.video,
            _access,
            _mediaKeys1,
            _mediaKeys2,
            _usingMediaKeys2 = false;;

        // Test MediaKeys assignment.
        assert_equals(_video.mediaKeys, null);
        assert_equals(typeof _video.setMediaKeys, 'function');

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            _access = access;
            return _access.createMediaKeys();
        }).then(test.step_func(function(result) {
            _mediaKeys1 = result;
            assert_not_equals(_mediaKeys1, null);
            // Create a second mediaKeys.
            return _access.createMediaKeys();
        })).then(test.step_func(function(result) {
            _mediaKeys2 = result;
            assert_not_equals(_mediaKeys2, null);
            // Set _mediaKeys1 on video.
            return _video.setMediaKeys(_mediaKeys1);
        })).then(test.step_func(function() {
            assert_equals(_video.mediaKeys, _mediaKeys1);
            // Set _mediaKeys2 on video (switching MediaKeys).
            return _video.setMediaKeys(_mediaKeys2);
        })).then(test.step_func(function() {
            assert_equals(_video.mediaKeys, _mediaKeys2);
            // Clear mediaKeys from video.
            return _video.setMediaKeys(null);
        })).then(test.step_func(function() {
            assert_equals(_video.mediaKeys, null);
            // Set _mediaKeys1 on video again.
            return _video.setMediaKeys(_mediaKeys1);
        })).then(test.step_func(function() {
            assert_equals(_video.mediaKeys, _mediaKeys1);
            return testmediasource(config);
        })).then(function(source) {
            // Set src attribute on Video Element
            _video.src = URL.createObjectURL(source);
            // According to the specification, support for changing the Media Keys object after
            // the src attribute on the video element has been set is optional. The following operation
            // may therefore either succeed or fail. We handle both cases.
            return _video.setMediaKeys(_mediaKeys2);
        }).then(test.step_func(function() {
            // Changing the Media Keys object succeeded
            _usingMediaKeys2 = true;
            assert_equals(_video.mediaKeys, _mediaKeys2);
            // Return something so the promise resolves properly.
            return Promise.resolve();
        }), test.step_func(function(error) {
            // Changing the Media Keys object failed
            _usingMediaKeys2 = false;
            assert_equals(_video.mediaKeys, _mediaKeys1);
            // The specification allows either NotSupportedError or InvalidStateError depending on
            // whether the failure was because changing Media Keys object is not supported
            // or just not allowed at this time, respectively.
            assert_in_array(error.name, ['InvalidStateError','NotSupportedError']);
            assert_not_equals(error.message, '');
            // Return something so the promise resolves properly.
            return Promise.resolve();
        })).then(function() {
            // According to the specification, support for clearing the Media Keys object associated
            // with the video element is optional. The following operation
            // may therefore either succeed or fail. We handle both cases.
            return _video.setMediaKeys(null);
        }).then(test.step_func(function() {
            // Clearing the media keys succeeded
            assert_equals(_video.mediaKeys, null);
            test.done();
        }), test.step_func(function(error) {
            // Clearing the media keys failed
            if(!_usingMediaKeys2) {
                assert_equals(_video.mediaKeys, _mediaKeys1);
            } else {
                assert_equals(_video.mediaKeys, _mediaKeys2);
            }
            // The specification allows either NotSupportedError or InvalidStateError depending on
            // whether the failure was because changing Media Keys object is not supported
            // or just not allowed at this time, respectively.
            assert_in_array(error.name, ['InvalidStateError','NotSupportedError']);
            assert_not_equals(error.message, '');
            test.done();
        })).catch(onFailure);
    }, testname);
}
