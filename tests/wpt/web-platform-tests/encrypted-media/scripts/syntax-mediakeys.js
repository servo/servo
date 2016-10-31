function runTest(config) {
    var keysystem = config.keysystem;
    var testname  = testnamePrefix(null, config.keysystem);
    var initDataType = config.initDataType;
    var configuration = {
        initDataTypes: [config.initDataType],
        audioCapabilities: [{contentType: config.audioType}],
        videoCapabilities: [{contentType: config.videoType}],
        sessionTypes: ['temporary']
    };

    function createMediaKeysAttributeTest() {
        return new Promise(function (resolve, reject) {
            var access;
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                assert_equals(typeof navigator.requestMediaKeySystemAccess, 'function');
                assert_true(isTypeSupported, "initDataType should be supported");
                return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
            }).then(function (result) {
                access = result;
                assert_equals(access.keySystem, keysystem);
                return access.createMediaKeys();
            }).then(function (mediaKeys) {
                assert_not_equals(mediaKeys, null);
                assert_equals(typeof mediaKeys, 'object');
                assert_equals(typeof mediaKeys.createSession, 'function');
                assert_equals(typeof mediaKeys.setServerCertificate, 'function');

                // Test creation of a second MediaKeys.
                // The extra parameter is ignored.
                return access.createMediaKeys('extra');
            }).then(function (mediaKeys) {
                assert_not_equals(mediaKeys, null);
                assert_equals(typeof mediaKeys, 'object');
                assert_equals(typeof mediaKeys.createSession, 'function');
                assert_equals(typeof mediaKeys.setServerCertificate, 'function');
                resolve();
            }).catch(function (error) {
                reject(error);
            });
        })
    }

    promise_test(function() {
        return createMediaKeysAttributeTest();
    }, testname + ' test MediaKeys attribute syntax');

    var kSetServerCertificateExceptionsTestCases = [
        // Too few parameters.
        {
            exception: 'TypeError',
            func: function (mk) {
                return mk.setServerCertificate();
            }
        },
        // Invalid parameters.
        {
            exception: 'TypeError',
            func: function (mk) {
                return mk.setServerCertificate('');
            }
        },
        {
            exception: 'TypeError',
            func: function (mk) {
                return mk.setServerCertificate(null);
            }
        },
        {
            exception: 'TypeError',
            func: function (mk) {
                return mk.setServerCertificate(undefined);
            }
        },
        {
            exception: 'TypeError',
            func: function (mk) {
                return mk.setServerCertificate(1);
            }
        },
        // Empty array.
        {
            exception: 'TypeError',
            func: function (mk) {
                return mk.setServerCertificate(new Uint8Array(0));
            }
        }
    ];


    function setServerCertificateTestExceptions() {
        return new Promise(function(resolve, reject) {
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                        assert_equals(typeof navigator.requestMediaKeySystemAccess, 'function');
                        assert_true(isTypeSupported, "initDataType not supported");
                        return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                    }).then(function (access) {
                        return access.createMediaKeys();
                    }).then(function (mediaKeys) {
                        var promises = kSetServerCertificateExceptionsTestCases.map(function (testCase) {
                            return test_exception(testCase, mediaKeys);
                        });
                        assert_not_equals(promises.length, 0);
                        return Promise.all(promises);
                    }).then(function () {
                        resolve();
                    }).catch(function (error) {
                        reject(error);
                    });
        })
    }
    promise_test(function() {
        return setServerCertificateTestExceptions();
    }, testname + ' test MediaKeys setServerCertificate() exceptions.');

    // All calls to |func| in this group resolve. setServerCertificate with these cert may either resolve with true
    // for clearkey or throw a DOMException.
    var kSetServerCertificateTestCases = [
        {
            // Pass in ArrayBufferView
            func: function (mk) {
                var cert = new Uint8Array(200);
                assert_true(ArrayBuffer.isView(cert));

                return new Promise(function (resolve, reject) {
                    mk.setServerCertificate(cert).then(function (value) {
                        resolve(value);
                    }).catch(function (error) {
                        if (Object.prototype.toString.call(error) === "[object DOMException]") {
                            resolve(false);
                        }
                    });
                })
            },
            expected: false
        },
        {
            // Pass in ArrayBuffer.
            func: function (mk) {
                var cert = new ArrayBuffer(200);
                assert_false(ArrayBuffer.isView(cert));
                return new Promise(function (resolve) {
                    mk.setServerCertificate(cert).then(function (resolveValue) {
                        resolve(resolveValue);
                    }).catch(function (error) {
                        if (Object.prototype.toString.call(error) === "[object DOMException]") {
                            resolve(false);
                        }
                    });
                })
            },
            expected: false
        }
    ];
    function setServerCertificateTest(){
        return new Promise(function(resolve, reject){
            var expected_result;
            isInitDataTypeSupported(keysystem, initDataType).then(function (isTypeSupported) {
                    assert_equals(typeof navigator.requestMediaKeySystemAccess, 'function');
                    assert_true(isTypeSupported, "initDataType not supported");
                    return navigator.requestMediaKeySystemAccess(keysystem, [configuration]);
                }).then(function (access) {
                    return access.createMediaKeys();
                }).then(function (mediaKeys) {
                    var promises = kSetServerCertificateTestCases.map(function (testCase) {
                        return testCase.func.call(null, mediaKeys);
                    });
                    expected_result = kSetServerCertificateTestCases.map(function (testCase) {
                        return testCase.expected;
                    });
                    assert_not_equals(promises.length, 0);
                    return Promise.all(promises);
                }).then(function (result) {
                    assert_array_equals(result, expected_result);
                    resolve();
                }).catch(function (error) {
                    reject(error);
                });
        })
    }
    promise_test(function() {
       return  setServerCertificateTest();
    }, testname + ' test MediaKeys setServerCertificate() syntax with non-empty certificate.');
}
