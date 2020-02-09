function runTest(config) {
    var keysystem = config.keysystem;
    var testname  = testnamePrefix(null, config.keysystem);
    var initDataType = config.initDataType;
    var initData = config.initData;
    var configuration = {
        initDataTypes: [config.initDataType],
        audioCapabilities: [{contentType: config.audioType}],
        videoCapabilities: [{contentType: config.videoType}],
        sessionTypes: ['temporary']
    };

    var kTypeSpecificGenerateRequestExceptionsTestCases = [
        // Tests in this set use a shortened parameter name due to
        // format_value() only returning the first 60 characters as the
        // result. With a longer name the first 60 characters is not
        // enough to determine which test failed. Even with the
        // shortened name, the error message for the last couple of
        // tests is the same.

        // Too few parameters.
        {
            exception: 'TypeError',
            func: function (mk1, type) {
                return mk1.createSession().generateRequest(type);
            }
        },
        // Invalid parameters.
        {
            exception: 'TypeError',
            func: function (mk2, type) {
                return mk2.createSession().generateRequest(type, '');
            }
        },
        {
            exception: 'TypeError',
            func: function (mk3, type) {
                return mk3.createSession().generateRequest(type, null);
            }
        },
        {
            exception: 'TypeError',
            func: function (mk4, type) {
                return mk4.createSession().generateRequest(type, undefined);
            }
        },
        {
            exception: 'TypeError',
            func: function (mk5, type) {
                return mk5.createSession().generateRequest(type, 1);
            }
        },
        // (new Uint8Array(0)) returns empty array. So 'TypeError' should
        // be returned.
        {
            exception: 'TypeError',
            func: function (mk6, type) {
                return mk6.createSession().generateRequest(type, new Uint8Array(0));
            }
        },
        // Using an empty type should return a 'TypeError'.
        {
            exception: 'TypeError',
            func: function (mk7, type) {
                return mk7.createSession().generateRequest('', initData);
            }
        },
    ];
    function generateRequestTestExceptions(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    var mp4SessionPromises = kTypeSpecificGenerateRequestExceptionsTestCases.map(function (testCase) {
                        return test_exception(testCase, mediaKeys, initDataType, initData);
                    });
                    assert_not_equals(mp4SessionPromises.length, 0);
                    return Promise.all(mp4SessionPromises);
                }).then(function (result) {
                    resolve();
                }).catch(function (error) {
                    reject(error);
                });
        })
    }
    promise_test(function() {
        return generateRequestTestExceptions();
    }, testname + ' test MediaKeySession generateRequest() exceptions.');

    var kLoadExceptionsTestCases = [
        // Too few parameters.
        {
            exception: 'TypeError',
            func: function (mk1) {
                return mk1.createSession('temporary').load();
            }
        },
        {
            exception: 'TypeError',
            func: function (mk3) {
                return mk3.createSession('temporary').load('');
            }
        },
        {
            exception: 'TypeError',
            func: function (mk4) {
                return mk4.createSession('temporary').load(1);
            }
        },
        {
            exception: 'TypeError',
            func: function (mk5) {
                return mk5.createSession('temporary').load('!@#$%^&*()');
            }
        },
        {
            exception: 'TypeError',
            func: function (mk6) {
                return mk6.createSession('temporary').load('1234');
            }
        }
    ];
    function loadTestExceptions(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    var sessionPromises = kLoadExceptionsTestCases.map(function (testCase) {
                        return test_exception(testCase, mediaKeys);
                    });
                    assert_not_equals(sessionPromises.length, 0);
                    return Promise.all(sessionPromises);
                }).then(function () {
                    resolve();
                }).catch(function (error) {
                   reject(error);
                });
        })
    }
    promise_test(function() {
        return loadTestExceptions();
    }, testname + ' test MediaKeySession load() exceptions.');

    // All calls to |func| in this group are supposed to succeed.
    // However, the spec notes that some things are optional for
    // Clear Key. In particular, support for persistent sessions
    // is optional. Since some implementations won't support some
    // features, a NotSupportedError is treated as a success
    // if |isNotSupportedAllowed| is true.
    var kCreateSessionTestCases = [
        // Use the default sessionType.
        {
            func: function(mk) { return mk.createSession(); },
            isNotSupportedAllowed: false
        },
        // Try variations of sessionType.
        {
            func: function(mk) { return mk.createSession('temporary'); },
            isNotSupportedAllowed: false
        },
        {
            func: function(mk) { return mk.createSession(undefined); },
            isNotSupportedAllowed: false
        },
        {
            // Since this is optional, some Clear Key implementations
            // will succeed, others will return a "NotSupportedError".
            // Both are allowed results.
            func: function(mk) { return mk.createSession('persistent-license'); },
            isNotSupportedAllowed: true
        },
        // Try additional parameter, which should be ignored.
        {
            func: function(mk) { return mk.createSession('temporary', 'extra'); },
            isNotSupportedAllowed: false
        }
    ];
    // This function checks that calling generateRequest() works for
    // various sessions. |testCase.func| creates a MediaKeySession
    // object, and then generateRequest() is called on that object. It
    // allows for an NotSupportedError to be generated and treated as a
    // success, if allowed. See comment above kCreateSessionTestCases.
    function test_generateRequest(testCase, mediaKeys, type, initData) {
        var mediaKeySession;
        try {
            mediaKeySession = testCase.func.call(null, mediaKeys);
        } catch (e) {
            assert_true(testCase.isNotSupportedAllowed);
            assert_equals(e.name, 'NotSupportedError');
            return Promise.resolve('not supported');
        }
        return mediaKeySession.generateRequest(type, initData);
    }
    function generateRequestForVariousSessions(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType should be supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    var mp4SessionPromises = kCreateSessionTestCases.map(function (testCase) {
                        return test_generateRequest(testCase, mediaKeys, initDataType, initData);
                    });
                    assert_not_equals(mp4SessionPromises.length, 0);
                    return Promise.all(mp4SessionPromises);
                }).then(function () {
                   resolve();
                }).catch(function (error) {
                   reject(error);
                });
        })
    }
    promise_test(function() {
        return generateRequestForVariousSessions();
    }, testname + ' test if MediaKeySession generateRequest() resolves for various sessions');

    var kUpdateSessionExceptionsTestCases = [
        // Tests in this set use a shortened parameter name due to
        // format_value() only returning the first 60 characters as the
        // result. With a longer name (mediaKeySession) the first 60
        // characters is not enough to determine which test failed.

        // Too few parameters.
        {
            exception: 'TypeError',
            func: function (s) {
                return s.update();
            }
        },
        // Invalid parameters.
        {
            exception: 'TypeError',
            func: function (s) {
                return s.update('');
            }
        },
        {
            exception: 'TypeError',
            func: function (s) {
                return s.update(null);
            }
        },
        {
            exception: 'TypeError',
            func: function (s) {
                return s.update(undefined);
            }
        },
        {
            exception: 'TypeError',
            func: function (s) {
                return s.update(1);
            }
        },
        // (new Uint8Array(0)) returns empty array. So 'TypeError' should
        // be returned.
        {
            exception: 'TypeError',
            func: function (s) {
                return s.update(new Uint8Array(0));
            }
        }
    ];

    function updateTestExceptions(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    var mp4SessionPromises = kUpdateSessionExceptionsTestCases.map(function (testCase) {
                        var mediaKeySession = mediaKeys.createSession();
                        return mediaKeySession.generateRequest(initDataType, initData).then(function (result) {
                            return test_exception(testCase, mediaKeySession);
                        });
                    });
                    assert_not_equals(mp4SessionPromises.length, 0);
                    return Promise.all(mp4SessionPromises);
                }).then(function () {
                    resolve();
                }).catch(function (error) {
                    reject(error);
                });
        })
    }
    promise_test(function() {
        return updateTestExceptions();
    }, testname + ' test MediaKeySession update() exceptions.');

    function create_close_exception_test(mediaKeys) {
        var mediaKeySession = mediaKeys.createSession();
        return mediaKeySession.close().then(function (result) {
                assert_unreached('close() should not succeed if session uninitialized');
            }).catch(function (error) {
                assert_equals(error.name, 'InvalidStateError');
                // Return something so the promise resolves.
                return Promise.resolve();
            });
    }
    function closeTestExceptions(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    return create_close_exception_test(mediaKeys);
                }).then(function () {
                    resolve();
                }).catch(function (error) {
                    reject(error);
                });
        });
    }
    promise_test(function() {
        return closeTestExceptions();
    }, testname + ' test MediaKeySession close() exceptions.');

    function create_remove_exception_test(mediaKeys, type, initData) {
        // remove() on an uninitialized session should fail.
        var mediaKeySession = mediaKeys.createSession('temporary');
        return mediaKeySession.remove().then(function (result) {
                assert_unreached('remove() should not succeed if session uninitialized');
            }, function (error) {
                assert_equals(error.name, 'InvalidStateError');
            });
    }
    function removeTestException(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    return create_remove_exception_test(mediaKeys, initDataType, initData);
                }).then(function () {
                    resolve();
                }).catch(function (error) {
                    reject(error);
                });
        });
    }
    promise_test(function() {
        return removeTestException();
    }, testname + ' test MediaKeySession remove() exceptions.');

    // All calls to |func| in this group are supposed to succeed.
    // However, the spec notes that some things are optional for
    // Clear Key. In particular, support for persistent sessions
    // is optional. Since some implementations won't support some
    // features, a NotSupportedError is treated as a success
    // if |isNotSupportedAllowed| is true.
    var kCreateSessionTestCases = [
        // Use the default sessionType.
        {
            func: function (mk) {
                return mk.createSession();
            },
            isNotSupportedAllowed: false
        },
        // Try variations of sessionType.
        {
            func: function (mk) {
                return mk.createSession('temporary');
            },
            isNotSupportedAllowed: false
        },
        {
            func: function (mk) {
                return mk.createSession(undefined);
            },
            isNotSupportedAllowed: false
        },
        {
            // Since this is optional, some Clear Key implementations
            // will succeed, others will return a "NotSupportedError".
            // Both are allowed results.
            func: function (mk) {
                return mk.createSession('persistent-license');
            },
            isNotSupportedAllowed: true
        },
        // Try additional parameter, which should be ignored.
        {
            func: function (mk) {
                return mk.createSession('temporary', 'extra');
            },
            isNotSupportedAllowed: false
        }
    ];

    // This function checks that calling |testCase.func| creates a
    // MediaKeySession object with some default values. It also
    // allows for an NotSupportedError to be generated and treated as a
    // success, if allowed. See comment above kCreateSessionTestCases.
    function test_createSession(testCase, mediaKeys) {
        var mediaKeySession;
        try {
            mediaKeySession = testCase.func.call(null, mediaKeys);
        } catch (e) {
            assert_true(testCase.isNotSupportedAllowed);
            return;
        }
        assert_equals(typeof mediaKeySession, 'object');
        assert_equals(typeof mediaKeySession.addEventListener, 'function');
        assert_equals(typeof mediaKeySession.sessionId, 'string');
        assert_equals(typeof mediaKeySession.expiration, 'number');
        assert_equals(typeof mediaKeySession.closed, 'object');
        assert_equals(typeof mediaKeySession.keyStatuses, 'object');
        assert_equals(typeof mediaKeySession.onkeystatuseschange, 'object');
        assert_equals(typeof mediaKeySession.onmessage, 'object');
        assert_equals(typeof mediaKeySession.generateRequest, 'function');
        assert_equals(typeof mediaKeySession.load, 'function');
        assert_equals(typeof mediaKeySession.update, 'function');
        assert_equals(typeof mediaKeySession.close, 'function');
        assert_equals(typeof mediaKeySession.remove, 'function');
        assert_equals(mediaKeySession.sessionId, '');
    }
    function createSessionTest(){
        return new Promise(function(resolve, reject){
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    kCreateSessionTestCases.map(function (testCase) {
                        test_createSession(testCase, mediaKeys);
                    });
                    resolve();
                }).catch(function (error) {
                    reject(error);
                });
        })
    }
    promise_test(function() {
        return createSessionTest();
    }, testname + ' test MediaKeySession attribute syntax.');


}
