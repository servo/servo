function runTest(config)
{
    // This test passes |response| to update() as a JSON Web Key Set.
    // CDMs other than Clear Key won't expect |response| in this format.
    promise_test(function(test) {
        var initDataType;
        var initData;
        var keySystem = config.keysystem;
        var mediaKeySession;

        function createReallyLongJWKSet()
        {
            // This is just a standard JWKSet with a lot of
            // extra items added to the end. Key ID and key
            // doesn't really matter.
            var jwkSet = '{"keys":[{'
                       +     '"kty":"oct",'
                       +     '"k":"MDEyMzQ1Njc4OTAxMjM0NQ",'
                       +     '"kid":"MDEyMzQ1Njc4OTAxMjM0NQ"'
                       + '}]';
            return jwkSet + ',"test":"unknown"'.repeat(4000) + '}';
        }

        var p = navigator.requestMediaKeySystemAccess(keySystem, getSimpleConfiguration()).then(function(access) {
            initDataType = access.getConfiguration().initDataTypes[0];
            initData = getInitData(initDataType);
            return access.createMediaKeys();
        }).then(function(mediaKeys) {
            mediaKeySession = mediaKeys.createSession();
            var eventWatcher = new EventWatcher(test, mediaKeySession, ['message']);
            var promise = eventWatcher.wait_for('message');
            mediaKeySession.generateRequest(initDataType, initData);
            return promise;
        }).then(function () {
            var jwkSet = createReallyLongJWKSet();
            assert_greater_than(jwkSet.length, 65536);
            var jwkSetArray = stringToUint8Array(jwkSet);
            return mediaKeySession.update(jwkSetArray);
        });

        return promise_rejects_js(test, TypeError, p);
    }, 'update() with invalid response (longer than 64Kb characters) should fail.');
}
