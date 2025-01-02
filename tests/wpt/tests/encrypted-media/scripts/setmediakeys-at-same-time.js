function runTest(config, qualifier) {
    var testname = testnamePrefix(qualifier, config.keysystem)
                                             + ', setmediakeys at same time';

    var configuration = getSimpleConfigurationForContent(config.content);

    async_test(function(test) {
        var _video = config.video,
            _access,
            _mediaKeys1,
            _mediaKeys2,
            _mediaKeys3,
            _mediaKeys4,
            _mediaKeys5;

        // Test MediaKeys assignment.
        assert_equals(_video.mediaKeys, null);
        assert_equals(typeof _video.setMediaKeys, 'function');

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function setMediaKeys(mediaKeys) {
            return _video.setMediaKeys(mediaKeys)
                .then(function() {return 1}, function() {return 0})
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            _access = access;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys1 = result;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys2 = result;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys3 = result;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys4 = result;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys5 = result;
            return Promise.all([
                setMediaKeys(_mediaKeys1),
                setMediaKeys(_mediaKeys2),
                setMediaKeys(_mediaKeys3),
                setMediaKeys(_mediaKeys4),
                setMediaKeys(_mediaKeys5)
            ]);
        }).then(function(results) {
            var sum = results.reduce((a, b) => a + b, 0);
            assert_in_array(sum,[1,5]);
            test.done();
        }).catch(onFailure);
    }, testname);
}