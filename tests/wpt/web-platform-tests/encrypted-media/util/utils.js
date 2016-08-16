function getInitData(initDataType) {
    if (initDataType == 'cenc') {
        return new Uint8Array([
            0x00, 0x00, 0x00, 0x00, // size = 0
            0x70, 0x73, 0x73, 0x68, // 'pssh'
            0x01, // version = 1
            0x00, 0x00, 0x00, // flags
            0x10, 0x77, 0xEF, 0xEC, 0xC0, 0xB2, 0x4D, 0x02, // Common SystemID
            0xAC, 0xE3, 0x3C, 0x1E, 0x52, 0xE2, 0xFB, 0x4B,
            0x00, 0x00, 0x00, 0x01, // key count
            0x00, 0x00, 0x00, 0x00, 0x03, 0xd2, 0xfc, 0x41, // key
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00 // datasize
        ]);
    }
    if (initDataType == 'keyids') {
        var keyId = new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x03, 0xd2, 0xfc, 0x41,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]);
        return stringToUint8Array(createKeyIDs(keyId));
    }
    throw 'initDataType ' + initDataType + ' not supported.';
}

function stringToUint8Array(str)
{
    var result = new Uint8Array(str.length);
    for(var i = 0; i < str.length; i++) {
        result[i] = str.charCodeAt(i);
    }
    return result;
}
// Encodes |data| into base64url string. There is no '=' padding, and the
// characters '-' and '_' must be used instead of '+' and '/', respectively.
function base64urlEncode(data) {
    var result = btoa(String.fromCharCode.apply(null, data));
    return result.replace(/=+$/g, '').replace(/\+/g, "-").replace(/\//g, "_");
}
// Decode |encoded| using base64url decoding.
function base64urlDecode(encoded) {
    return atob(encoded.replace(/\-/g, "+").replace(/\_/g, "/"));
}
// Decode |encoded| using base64 to a Uint8Array
function base64DecodeToUnit8Array(encoded) {
    return new Uint8Array( atob( encoded ).split('').map( function(c){return c.charCodeAt(0);} ) );
}
// Clear Key can also support Key IDs Initialization Data.
// ref: http://w3c.github.io/encrypted-media/keyids-format.html
// Each parameter is expected to be a key id in an Uint8Array.
function createKeyIDs() {
    var keyIds = '{"kids":["';
    for (var i = 0; i < arguments.length; i++) {
        if (i != 0) keyIds += '","';
        keyIds += base64urlEncode(arguments[i]);
    }
    keyIds += '"]}';
    return keyIds;
}

function getSupportedKeySystem() {
    var userAgent = navigator.userAgent.toLowerCase();
    var keysystem = undefined;
    if(userAgent.indexOf('chrome') > -1) {
        keysystem = 'com.widevine.alpha';
    }

    return keysystem;
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

