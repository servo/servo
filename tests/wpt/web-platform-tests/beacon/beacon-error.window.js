// META: script=/common/utils.js
// META: script=beacon-common.sub.js

"use strict";

test(function() {
    // Payload that should cause sendBeacon to return false because it exceeds the maximum payload size.
    var exceedPayload = Array(maxPayloadSize + 1).fill('z').join("");

    var success = navigator.sendBeacon("http://doesnotmatter", exceedPayload);
    assert_false(success, "calling 'navigator.sendBeacon()' with payload size exceeding the maximum size must fail");
}, "Verify calling 'navigator.sendBeacon()' with a large payload returns 'false'.");

test(function() {
    var invalidUrl = "http://invalid:url";
    assert_throws(new TypeError(), function() { navigator.sendBeacon(invalidUrl, smallPayload); },
        `calling 'navigator.sendBeacon()' with an invalid URL '${invalidUrl}' must throw a TypeError`);
}, "Verify calling 'navigator.sendBeacon()' with an invalid URL throws an exception.");

test(function() {
    var invalidUrl = "nothttp://invalid.url";
    assert_throws(new TypeError(), function() { navigator.sendBeacon(invalidUrl, smallPayload); },
         `calling 'navigator.sendBeacon()' with a non-http(s) URL '${invalidUrl}' must throw a TypeError`);
}, "Verify calling 'navigator.sendBeacon()' with a URL that is not a http(s) scheme throws an exception.");

// We'll validate that we can send one beacon that uses our entire Quota and then fail to send one that is just one char.
test(function () {
    var destinationURL = "/fetch/api/resources/trickle.py?count=1&ms=1000";

    var firstSuccess = navigator.sendBeacon(destinationURL, maxPayload);
    assert_true(firstSuccess, "calling 'navigator.sendBeacon()' with our max payload size should succeed.");

    // Now we'll send just one character.
    var secondSuccess = navigator.sendBeacon(destinationURL, "1");
    assert_false(secondSuccess, "calling 'navigator.sendBeacon()' with just one char should fail while our Quota is used up.");

}, "Verify calling 'navigator.sendBeacon()' with a small payload fails while Quota is completely utilized.");

done();
