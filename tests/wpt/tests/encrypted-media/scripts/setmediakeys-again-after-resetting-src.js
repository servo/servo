function runTest(config, qualifier) {
    var testname = testnamePrefix(qualifier, config.keysystem)
                                     + ', setmediakeys again after resetting src';

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
            config.messagehandler(event.messageType, event.message).then(function(response) {
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

        function playVideoAndWaitForTimeupdate()
        {
            return new Promise(function(resolve) {
                testmediasource(config).then(function(source) {
                    _mediaKeySession = _mediaKeys.createSession('temporary');
                    _video.src = URL.createObjectURL(source);
                });
                _video.addEventListener('timeupdate', function listener(event) {
                    if (event.target.currentTime < (config.duration || 1))
                        return;
                    _video.removeEventListener('timeupdate', listener);
                    resolve('success');
                });
            });
        }

        waitForEventAndRunStep('encrypted', _video, onEncrypted, test);
        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            _access = access;
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys = result;
            return _video.setMediaKeys(_mediaKeys);
        }).then(function() {
            return config.servercertificate ? _mediaKeys.setServerCertificate( config.servercertificate ) : true;
        }).then(function( success ) {
            return playVideoAndWaitForTimeupdate();
        }).then(function(results) {
            return _access.createMediaKeys();
        }).then(function(result) {
            _mediaKeys = result;
            _video.src = '';
            return _video.setMediaKeys(_mediaKeys);
        }).then(function() {
            return config.servercertificate ? _mediaKeys.setServerCertificate( config.servercertificate ) : true;
        }).then(function( success ) {
            return playVideoAndWaitForTimeupdate();
        }).then(function() {
            _video.src = '';
            test.done();
        }).catch(onFailure);
    }, testname);
}