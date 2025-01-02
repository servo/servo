function runTest(config,qualifier)
{
    var testname = testnamePrefix(qualifier, config.keysystem) + ', temporary, keystatuses';

    var configuration = getSimpleConfigurationForContent(config.content);

    if (config.initDataType && config.initData) {
        configuration.initDataTypes = [config.initDataType];
    }

    async_test(function(test)
    {
        var mediaKeySession;
        var initDataType;
        var initData;
        var closed = false;

        // Even though key ids are uint8, using printable values so that
        // they can be verified easily.
        var key1 = new Uint8Array(config.content.keys[0].kid),
            key2 = new Uint8Array(config.content.keys[1].kid),
            key1String = arrayBufferAsString(key1),
            key2String = arrayBufferAsString(key2);

        function onFailure(error) {
            forceTestFailureFromPromise(test, error);
        }

        function processMessage(event)
        {
            // No keys added yet.
            assert_equals(mediaKeySession.keyStatuses.size, 0);

            waitForEventAndRunStep('keystatuseschange', mediaKeySession, processKeyStatusesChange, test);

            // Add keys to session
            config.messagehandler(event.messageType, event.message).then(function(response) {
                return event.target.update(response);
            }).catch(onFailure);
        }

        function checkKeyStatusFor2Keys()
        {
            // Two keys added, so both should show up in |keyStatuses|.
            assert_equals(mediaKeySession.keyStatuses.size, 2);

            // Check |keyStatuses| for 2 entries.
            var result = [];
            for (let item of mediaKeySession.keyStatuses) {
                result.push({ key: arrayBufferAsString(item[0]), value: item[1] });
            }
            function lexicographical( a, b ) { return a < b ? -1 : a === b ? 0 : +1; }
            function lexicographicalkey( a, b ) { return lexicographical( a.key, b.key ); }
            var expected1 = [{ key: key1String, value: 'usable'}, { key: key2String, value: 'usable'}].sort( lexicographicalkey );
            var expected2 = [{ key: key1String, value: 'status-pending'}, { key: key2String, value: 'status-pending'}].sort( lexicographicalkey );
            assert_in_array(    JSON.stringify(result),
                                [ JSON.stringify(expected1),JSON.stringify(expected2) ],
                                 "keystatuses should have the two expected keys with keystatus 'usable' or 'status-pending'");

            // |keyStatuses| must contain both keys.
            result = [];
            for (var key of mediaKeySession.keyStatuses.keys()) {
                result.push(arrayBufferAsString(key));
            }
            assert_array_equals(result,
                                [key1String, key2String].sort( lexicographical ),
                                "keyStatuses.keys() should return an iterable over the two expected keys");

            // Both values in |mediaKeySession| should be 'usable' or 'status-pending'.
            result = [];
            for (var value of mediaKeySession.keyStatuses.values()) {
                result.push(value);
            }

            assert_equals( result.length, 2, "keyStatuses.values() should have two elements" );
            assert_equals( result[0], result[1], "the values in keyStatuses.values() should be equal" );
            assert_in_array( result[0], [ 'usable', 'status-pending' ] );

            // Check |keyStatuses.entries()|.
            result = [];
            for (var entry of mediaKeySession.keyStatuses.entries()) {
                result.push({ key: arrayBufferAsString(entry[0]), value: entry[1] });
            }
            assert_in_array(JSON.stringify(result),
                            [ JSON.stringify(expected1), JSON.stringify(expected2) ],
                                 "keyStatuses.entries() should return an iterable over the two expected keys, with keystatus 'usable' or 'status-pending'");

            // forEach() should return both entries.
            result = [];
            mediaKeySession.keyStatuses.forEach(function(status, keyId) {
                result.push({ key: arrayBufferAsString(keyId), value: status });
            });
            assert_in_array(JSON.stringify(result),
                            [ JSON.stringify(expected1), JSON.stringify(expected2) ],
                                 "keyStatuses.forEach() should iterate over the two expected keys, with keystatus 'usable' or 'status-pending'");

            // has() and get() should return the expected values.
            assert_true(mediaKeySession.keyStatuses.has(key1), "keyStatuses should have key1");
            assert_true(mediaKeySession.keyStatuses.has(key2), "keyStatuses should have key2");
            assert_in_array(mediaKeySession.keyStatuses.get(key1), [ 'usable', 'status-pending' ], "key1 should have status 'usable' or 'status-pending'");
            assert_in_array(mediaKeySession.keyStatuses.get(key2), [ 'usable', 'status-pending' ], "key2 should have status 'usable' or 'status-pending'");

            // Try some invalid keyIds.
            var invalid1 = key1.subarray(0, key1.length - 1);
            assert_false(mediaKeySession.keyStatuses.has(invalid1), "keystatuses should not have invalid key (1)");
            assert_equals(mediaKeySession.keyStatuses.get(invalid1), undefined, "keystatus value for invalid key should be undefined (1)");

            var invalid2 = key1.subarray(1);
            assert_false(mediaKeySession.keyStatuses.has(invalid2), "keystatuses should not have invalid key (2)");
            assert_equals(mediaKeySession.keyStatuses.get(invalid2), undefined, "keystatus value for invalid key should be undefined (2)");

            var invalid3 = new Uint8Array(key1);
            invalid3[0] += 1;
            assert_false(mediaKeySession.keyStatuses.has(invalid3), "keystatuses should not have invalid key (3)");
            assert_equals(mediaKeySession.keyStatuses.get(invalid3), undefined, "keystatus value for invalid key should be undefined (3)");

            var invalid4 = new Uint8Array(key1);
            invalid4[invalid4.length - 1] -= 1;
            assert_false(mediaKeySession.keyStatuses.has(invalid4), "keystatuses should not have invalid key (4)");
            assert_equals(mediaKeySession.keyStatuses.get(invalid4), undefined, "keystatus value for invalid key should be undefined (4)");

            var invalid5 = new Uint8Array(key1.length + 1);
            invalid5.set(key1, 1);  // First element will be 0.
            assert_false(mediaKeySession.keyStatuses.has(invalid5), "keystatuses should not have invalid key (5)");
            assert_equals(mediaKeySession.keyStatuses.get(invalid5), undefined, "keystatus value for invalid key should be undefined (5)");

            var invalid6 = new Uint8Array(key1.length + 1);
            invalid6.set(key1, 0);  // Last element will be 0.
            assert_false(mediaKeySession.keyStatuses.has(invalid6), "keystatuses should not have invalid key (6)");
            assert_equals(mediaKeySession.keyStatuses.get(invalid6), undefined, "keystatus value for invalid key should be undefined (6)");
        }

        function processKeyStatusesChange(event)
        {
            if (!closed)
            {
                // The first keystatuseschange (caused by update())
                // should include both keys.
                checkKeyStatusFor2Keys();

                mediaKeySession.close().catch(onFailure);
                closed = true;
            }
            else
            {
                // The second keystatuseschange (caused by close())
                // should not have any keys.
                assert_equals(mediaKeySession.keyStatuses.size, 0);
                test.done();
            }
        }

        navigator.requestMediaKeySystemAccess(config.keysystem, [configuration]).then(function(access) {
            return access.createMediaKeys();
        }).then(test.step_func(function(mediaKeys) {
            mediaKeySession = mediaKeys.createSession();

            // There should be no keys defined yet.
            //verifyKeyStatuses(mediaKeySession.keyStatuses, { expected: [], unexpected: [key1, key2] });

            waitForEventAndRunStep('message', mediaKeySession, processMessage, test);
            return mediaKeySession.generateRequest(config.initDataType, config.initData);
        })).catch(onFailure);
    }, testname );
}
