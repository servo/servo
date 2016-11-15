function runTest(config, qualifier) {
    var testname = testnamePrefix(qualifier, config.keysystem)
                                    + ', setmediakeys again after playback';

    var configuration = getSimpleConfigurationForContent(config.content);

    if (config.initDataType && config.initData) {
        configuration.initDataTypes = [config.initDataType];
    }

    async_test(function(test) {
        var _video = config.video,
            _access,
            _mediaKeys,
            _mediaKeySession,
            _mediaSource;

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function onMessage(event) {
            config.messagehandler(event.messageType, event.message).then( function(response) {
                _mediaKeySession.update(response).catch(onFailure).then(function() {
                    _video.play();
                });
            });
        }

        function onEncrypted(event) {
            waitForEventAndRunStep('message', _mediaKeySession, onMessage, test);
            _mediaKeySession.generateRequest(   config.initData ? config.initDataType : event.initDataType,
                                                config.initData || event.initData )
            .catch(onFailure);
        }

        function playVideo()
        {
            return new Promise(function(resolve) {
                _mediaKeySession = _mediaKeys.createSession('temporary');
                waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
                _video.src = URL.createObjectURL(_mediaSource);
                resolve('success');
            });
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            _access = access;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys = result;
            return _video.setMediaKeys(_mediaKeys);
        }).then(function() {
            return config.servercertificate ? _mediaKeys.setServerCertificate( config.servercertificate ) : true;
        }).then(function( success ) {
            return testmediasource(config);
        }).then(function(source) {
            _mediaSource = source;
            return playVideo();
        }).then(function(results) {
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys = result;
            return waitForEvent('playing', _video);
        }).then(test.step_func(function(result) {
            assert_false(_video.ended);
            return _video.setMediaKeys(_mediaKeys);
        })).then(function() {
            // Able to change MediaKeys while playing.
            // This is not required to fail.
            _video.src='';
            test.done();
        }, test.step_func(function(error) {
            assert_in_array(error.name, ['InvalidStateError','NotSupportedError']);
            _video.src='';
            test.done();
        })).catch(onFailure);
    }, testname);
}