function runTest(config, qualifier) {
    var testname = testnamePrefix(qualifier, config.keysystem)
                                 + ', setMediaKeys to multiple video elements';

    var configuration = getSimpleConfigurationForContent(config.content);

    if ( config.initDataType && config.initData ) {
        configuration.initDataTypes = [ config.initDataType ];
    }

    async_test (function (test) {
        var _video1 = config.video1,
            _video2 = config.video2,
            _mediaKeys;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            assert_equals(access.keySystem, config.keysystem)
            return access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys = result;
            assert_not_equals(_mediaKeys, null);
            assert_equals(typeof _mediaKeys.createSession, 'function');
            return _video1.setMediaKeys(_mediaKeys);
        }).then(function(result) {
            assert_not_equals(_video1.mediaKeys, null);
            assert_equals(_video1.mediaKeys, _mediaKeys);
            // The specification allows this to fail, but it is not required to fail.
            return _video2.setMediaKeys(_mediaKeys);
        }).then(function(result) {
            // Second setMediaKeys is not required to fail.
            assert_equals(_video2.mediaKeys, _mediaKeys);
            return Promise.resolve();
        }, function(error) {
            assert_equals(error.name, 'QuotaExceededError');
            assert_not_equals(error.message, '');
            // Return something so the promise resolves properly.
            return Promise.resolve();
        }).then(function() {
            // Now clear it from video1.
            return _video1.setMediaKeys(null);
        }).then(function() {
            // Should be assignable to video2.
            return _video2.setMediaKeys(_mediaKeys);
        }).then(function(result) {
            assert_not_equals(_video2.mediaKeys, null);
            assert_equals(_video2.mediaKeys, _mediaKeys);
            test.done();
        }).catch(onFailure);
    }, testname);
}