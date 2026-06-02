function runTest(config, qualifier) {
    var testname = testnamePrefix( qualifier, config.keysystem )
                         + ', setmediakeys multiple times with the same mediakeys';

    var configuration = getSimpleConfigurationForContent( config.content );

    if ( config.initDataType && config.initData ) {
        configuration.initDataTypes = [ config.initDataType ];
    }

    async_test (function (test) {
        var _video = config.video,
            _mediaKeys;

        // Test MediaKeys assignment.
        assert_equals(_video.mediaKeys, null);
        assert_equals(typeof _video.setMediaKeys, 'function');

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys = result;
            // Set mediaKeys for first time on video should work.
            return _video.setMediaKeys(_mediaKeys);
        }).then(function(result) {
            assert_equals(_video.mediaKeys, _mediaKeys);
            // Set mediaKeys on video again should return a resolved promise.
            return _video.setMediaKeys(_mediaKeys);
        }).then(function (result) {
            assert_equals(_video.mediaKeys, _mediaKeys);
            return testmediasource(config);
        }).then(function(source) {
            // Set src attribute on Video Element
            _video.src = URL.createObjectURL(source);
            // Set mediaKeys again on video should still return a resolved promise.
            return _video.setMediaKeys(_mediaKeys);
        }).then(function() {
            assert_equals(_video.mediaKeys, _mediaKeys);
            test.done();
        }).catch(onFailure);
    }, testname);
}