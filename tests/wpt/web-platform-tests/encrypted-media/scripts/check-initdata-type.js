 function runTest( config )
 {
    function checkInitDataType(initDataType)
    {
        return isInitDataTypeSupported(initDataType).then(function(result) {
            // If |initDataType| is not supported, simply succeed.
            if (!result)
                return Promise.resolve('Not supported');

            return navigator.requestMediaKeySystemAccess( config.keysystem, getSimpleConfigurationForInitDataType(initDataType))
                .then(function(access) {
                    return access.createMediaKeys();
                }).then(function(mediaKeys) {
                    var mediaKeySession = mediaKeys.createSession();
                    var initData = getInitData(initDataType);
                    return mediaKeySession.generateRequest(initDataType, initData);
                });
        });
    }

    promise_test(function()
    {
        return checkInitDataType('webm');
    }, config.keysystem + ' support for "webm".');

    promise_test(function()
    {
        return checkInitDataType('cenc');
    }, config.keysystem + ' support for "cenc".');

    promise_test(function()
    {
        return checkInitDataType('keyids');
    }, config.keysystem + ' support for "keyids".');
}