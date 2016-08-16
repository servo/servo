<!-- Copyright © 2016 Chromium authors and World Wide Web Consortium, (Massachusetts Institute of Technology, ERCIM, Keio University, Beihang). -->

var consoleDiv = null;

function consoleWrite(text)
{
    if (!consoleDiv && document.body) {
        consoleDiv = document.createElement('div');
        document.body.appendChild(consoleDiv);
    }
    var span = document.createElement('span');
    span.appendChild(document.createTextNode(text));
    span.appendChild(document.createElement('br'));
    consoleDiv.appendChild(span);
}

// Returns a promise that is fulfilled with true if |initDataType| is supported,
// or false if not.
function isInitDataTypeSupported(initDataType)
{
    return navigator.requestMediaKeySystemAccess(
                        "org.w3.clearkey", getSimpleConfigurationForInitDataType(initDataType))
        .then(function() { return true; }, function() { return false; });
}

function getInitData(initDataType)
{
  if (initDataType == 'webm') {
      return new Uint8Array([
          0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
      ]);
  }

  if (initDataType == 'cenc') {
      return new Uint8Array([
          0x00, 0x00, 0x00, 0x00,                          // size = 0
          0x70, 0x73, 0x73, 0x68,                          // 'pssh'
          0x01,                                            // version = 1
          0x00, 0x00, 0x00,                                // flags
          0x10, 0x77, 0xEF, 0xEC, 0xC0, 0xB2, 0x4D, 0x02,  // Common SystemID
          0xAC, 0xE3, 0x3C, 0x1E, 0x52, 0xE2, 0xFB, 0x4B,
          0x00, 0x00, 0x00, 0x01,                          // key count
          0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,  // key
          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
          0x00, 0x00, 0x00, 0x00                           // datasize
     ]);
  }

  if (initDataType == 'keyids') {
      var keyId = new Uint8Array([
          0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
      ]);
      return stringToUint8Array(createKeyIDs(keyId));
  }

  throw 'initDataType ' + initDataType + ' not supported.';
}

// Returns an array of audioCapabilities that includes entries for a set of
// codecs that should cover all user agents.
function getPossibleAudioCapabilities()
{
    return [
        { contentType: 'audio/mp4; codecs="mp4a.40.2"' },
        { contentType: 'audio/webm; codecs="opus"' },
    ];
}

// Returns a trivial MediaKeySystemConfiguration that should be accepted,
// possibly as a subset of the specified capabilities, by all user agents.
function getSimpleConfiguration()
{
    return [ {
        initDataTypes : [ 'webm', 'cenc', 'keyids' ],
        audioCapabilities: getPossibleAudioCapabilities()
    } ];
}

// Returns a MediaKeySystemConfiguration for |initDataType| that should be
// accepted, possibly as a subset of the specified capabilities, by all
// user agents.
function getSimpleConfigurationForInitDataType(initDataType)
{
    return [ {
        initDataTypes: [ initDataType ],
        audioCapabilities: getPossibleAudioCapabilities()
    } ];
}

// Returns a MediaKeySystemConfiguration for |mediaFile| that specifies
// both audio and video capabilities for the specified file..
function getConfigurationForFile(mediaFile)
{
    if (mediaFile.toLowerCase().endsWith('webm')) {
        return [ {
            initDataTypes: [ 'webm' ],
            audioCapabilities: [ { contentType: 'audio/webm; codecs="opus"' } ],
            videoCapabilities: [ { contentType: 'video/webm; codecs="vp8"' } ]
        } ];
    }

    // NOTE: Supporting other mediaFormats is not currently implemented as
    // Chromium only tests with WebM files.
    throw 'mediaFile ' + mediaFile + ' not supported.';
}

function waitForEventAndRunStep(eventName, element, func, stepTest)
{
    var eventCallback = function(event) {
        if (func)
            func(event);
    }
    if (stepTest)
        eventCallback = stepTest.step_func(eventCallback);

    element.addEventListener(eventName, eventCallback, true);
}

// Copied from LayoutTests/resources/js-test.js.
// See it for details of why this is necessary.
function asyncGC(callback)
{
    GCController.collectAll();
    setTimeout(callback, 0);
}

function createGCPromise()
{
    // Run gc() as a promise.
    return new Promise(
        function(resolve, reject) {
            asyncGC(resolve);
        });
}

function delayToAllowEventProcessingPromise()
{
    return new Promise(
        function(resolve, reject) {
            setTimeout(resolve, 0);
        });
}

function stringToUint8Array(str)
{
    var result = new Uint8Array(str.length);
    for(var i = 0; i < str.length; i++) {
        result[i] = str.charCodeAt(i);
    }
    return result;
}

function arrayBufferAsString(buffer)
{
    // MediaKeySession.keyStatuses iterators return an ArrayBuffer,
    // so convert it into a printable string.
    return String.fromCharCode.apply(null, new Uint8Array(buffer));
}

function dumpKeyStatuses(keyStatuses)
{
    consoleWrite("for (var entry of keyStatuses)");
    for (var entry of keyStatuses) {
        consoleWrite(arrayBufferAsString(entry[0]) + ": " + entry[1]);
    }
    consoleWrite("for (var keyId of keyStatuses.keys())");
    for (var keyId of keyStatuses.keys()) {
        consoleWrite(arrayBufferAsString(keyId));
    }
    consoleWrite("for (var status of keyStatuses.values())");
    for (var status of keyStatuses.values()) {
        consoleWrite(status);
    }
    consoleWrite("for (var entry of keyStatuses.entries())");
    for (var entry of keyStatuses.entries()) {
        consoleWrite(arrayBufferAsString(entry[0]) + ": " + entry[1]);
    }
    consoleWrite("keyStatuses.forEach()");
    keyStatuses.forEach(function(status, keyId) {
        consoleWrite(arrayBufferAsString(keyId) + ": " + status);
    });
}

// Verify that |keyStatuses| contains just the keys in |keys.expected|
// and none of the keys in |keys.unexpected|. All keys should have status
// 'usable'. Example call: verifyKeyStatuses(mediaKeySession.keyStatuses,
// { expected: [key1], unexpected: [key2] });
function verifyKeyStatuses(keyStatuses, keys)
{
    var expected = keys.expected || [];
    var unexpected = keys.unexpected || [];

    // |keyStatuses| should have same size as number of |keys.expected|.
    assert_equals(keyStatuses.size, expected.length);

    // All |keys.expected| should be found.
    expected.map(function(key) {
        assert_true(keyStatuses.has(key));
        assert_equals(keyStatuses.get(key), 'usable');
    });

    // All |keys.unexpected| should not be found.
    unexpected.map(function(key) {
        assert_false(keyStatuses.has(key));
        assert_equals(keyStatuses.get(key), undefined);
    });
}

// Encodes |data| into base64url string. There is no '=' padding, and the
// characters '-' and '_' must be used instead of '+' and '/', respectively.
function base64urlEncode(data)
{
    var result = btoa(String.fromCharCode.apply(null, data));
    return result.replace(/=+$/g, '').replace(/\+/g, "-").replace(/\//g, "_");
}

// Decode |encoded| using base64url decoding.
function base64urlDecode(encoded)
{
    return atob(encoded.replace(/\-/g, "+").replace(/\_/g, "/"));
}

// For Clear Key, the License Format is a JSON Web Key (JWK) Set, which contains
// a set of cryptographic keys represented by JSON. These helper functions help
// wrap raw keys into a JWK set.
// See:
// https://w3c.github.io/encrypted-media/#clear-key-license-format
// http://tools.ietf.org/html/draft-ietf-jose-json-web-key
//
// Creates a JWK from raw key ID and key.
// |keyId| and |key| are expected to be ArrayBufferViews, not base64-encoded.
function createJWK(keyId, key)
{
    var jwk = '{"kty":"oct","alg":"A128KW","kid":"';
    jwk += base64urlEncode(keyId);
    jwk += '","k":"';
    jwk += base64urlEncode(key);
    jwk += '"}';
    return jwk;
}

// Creates a JWK Set from multiple JWKs.
function createJWKSet()
{
    var jwkSet = '{"keys":[';
    for (var i = 0; i < arguments.length; i++) {
        if (i != 0)
            jwkSet += ',';
        jwkSet += arguments[i];
    }
    jwkSet += ']}';
    return jwkSet;
}

// Clear Key can also support Key IDs Initialization Data.
// ref: http://w3c.github.io/encrypted-media/keyids-format.html
// Each parameter is expected to be a key id in an Uint8Array.
function createKeyIDs()
{
    var keyIds = '{"kids":["';
    for (var i = 0; i < arguments.length; i++) {
        if (i != 0)
            keyIds += '","';
        keyIds += base64urlEncode(arguments[i]);
    }
    keyIds += '"]}';
    return keyIds;
}

function forceTestFailureFromPromise(test, error, message)
{
    // Promises convert exceptions into rejected Promises. Since there is
    // currently no way to report a failed test in the test harness, errors
    // are reported using force_timeout().
    if (message)
        consoleWrite(message + ': ' + error.message);
    else if (error)
        consoleWrite(error);

    test.force_timeout();
    test.done();
}

function extractSingleKeyIdFromMessage(message)
{
    var json = JSON.parse(String.fromCharCode.apply(null, new Uint8Array(message)));
    // Decode the first element of 'kids'.
    assert_equals(1, json.kids.length);
    var decoded_key = base64urlDecode(json.kids[0]);
    // Convert to an Uint8Array and return it.
    return stringToUint8Array(decoded_key);
}

// Create a MediaKeys object for Clear Key with 1 session. KeyId and key
// required for the video are already known and provided. Returns a promise
// that resolves to the MediaKeys object created.
function createMediaKeys(keyId, key)
{
    var mediaKeys;
    var mediaKeySession;
    var request = stringToUint8Array(createKeyIDs(keyId));
    var jwkSet = stringToUint8Array(createJWKSet(createJWK(keyId, key)));

    return navigator.requestMediaKeySystemAccess('org.w3.clearkey', getSimpleConfigurationForInitDataType('keyids')).then(function(access) {
        return access.createMediaKeys();
    }).then(function(result) {
        mediaKeys = result;
        mediaKeySession = mediaKeys.createSession();
        return mediaKeySession.generateRequest('keyids', request);
    }).then(function() {
        return mediaKeySession.update(jwkSet);
    }).then(function() {
        return Promise.resolve(mediaKeys);
    });
}

// Play the specified |content| on |video|. Returns a promise that is resolved
// after the video plays for |duration| seconds.
function playVideoAndWaitForTimeupdate(video, content, duration)
{
    video.src = content;
    video.play();
    return new Promise(function(resolve) {
        video.addEventListener('timeupdate', function listener(event) {
            if (event.target.currentTime < duration)
                return;
            video.removeEventListener('timeupdate', listener);
            resolve('success');
        });
    });
}
