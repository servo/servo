function runTest(config,qualifier) {
    var tests = [ ], initData, keyId;
    function push_test(keysystem, initDataType, initData, testname) {
        tests.push({ keysystem: keysystem, initDataType: initDataType, initData: initData, testname: testname });
    }

    initData = new Uint8Array(70000);
    push_test(config.keysystem, 'webm', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, webm, initData longer than 64Kb characters');

    initData = new Uint8Array(70000);
    push_test(config.keysystem, 'cenc', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, cenc, initData longer than 64Kb characters');

    initData = new Uint8Array(70000);
    push_test(config.keysystem, 'keyids', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, keyids, initData longer than 64Kb characters');

    // Invalid 'pssh' box as the size specified is larger than what
    // is provided.
    initData = new Uint8Array([
        0x00, 0x00, 0xff, 0xff,                          // size = huge
        0x70, 0x73, 0x73, 0x68,                          // 'pssh'
        0x00,                                            // version = 0
        0x00, 0x00, 0x00,                                // flags
        0x10, 0x77, 0xEF, 0xEC, 0xC0, 0xB2, 0x4D, 0x02,  // Common SystemID
        0xAC, 0xE3, 0x3C, 0x1E, 0x52, 0xE2, 0xFB, 0x4B,
        0x00, 0x00, 0x00, 0x00                           // datasize
    ]);
    push_test(config.keysystem, 'cenc', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, cenc, invalid initdata (invalid pssh)');

    // Invalid data as type = 'psss'.
    initData = new Uint8Array([
        0x00, 0x00, 0x00, 0x00,                          // size = 0
        0x70, 0x73, 0x73, 0x73,                          // 'psss'
        0x00,                                            // version = 0
        0x00, 0x00, 0x00,                                // flags
        0x10, 0x77, 0xEF, 0xEC, 0xC0, 0xB2, 0x4D, 0x02,  // Common SystemID
        0xAC, 0xE3, 0x3C, 0x1E, 0x52, 0xE2, 0xFB, 0x4B,
        0x00, 0x00, 0x00, 0x00                           // datasize
    ]);
    push_test(config.keysystem, 'cenc', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, cenc, invalid initdata (not pssh)');

    // Valid key ID size must be at least 1 character for keyids.
    keyId = new Uint8Array(0);
    initData = stringToUint8Array(createKeyIDs(keyId));
    push_test(config.keysystem, 'keyids', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, keyids, invalid initdata (too short key ID)');

    // Valid key ID size must be less than 512 characters for keyids.
    keyId = new Uint8Array(600);
    initData = stringToUint8Array(createKeyIDs(keyId));
    push_test(config.keysystem, 'keyids', initData, testnamePrefix( qualifier, config.keysystem ) + ', temporary, keyids, invalid initdata (too long key ID)');

    Promise.all( tests.map(function(testspec) {
        return isInitDataTypeSupported(testspec.keysystem,testspec.initDataType);
    })).then(function(results) {
        tests.filter(function(testspec, i) { return results[i]; } ).forEach(function(testspec) {
            promise_test(function(test) {
                // Create a "temporary" session for |keysystem| and call generateRequest()
                // with the provided initData. generateRequest() should fail with an
                // TypeError. Returns a promise that is resolved
                // if the error occurred and rejected otherwise.
                return navigator.requestMediaKeySystemAccess(testspec.keysystem, getSimpleConfigurationForInitDataType(testspec.initDataType)).then(function(access) {
                    return access.createMediaKeys();
                }).then(function(mediaKeys) {
                    var mediaKeySession = mediaKeys.createSession("temporary");
                    return mediaKeySession.generateRequest(testspec.initDataType, testspec.initData);
                }).then(test.step_func(function() {
                    assert_unreached('generateRequest() succeeded unexpectedly');
                }), test.step_func(function(error) {
                    assert_equals(error.name, 'TypeError');
                }));
            },testspec.testname);
        });
    });
}
