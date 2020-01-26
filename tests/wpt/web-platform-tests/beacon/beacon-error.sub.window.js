// META: script=/common/utils.js
// META: script=beacon-common.sub.js

"use strict";

test(function() {
    // Payload that should cause sendBeacon to return false because it exceeds the maximum payload size.
    var exceedPayload = Array(maxPayloadSize + 1).fill('z').join("");

    var success = navigator.sendBeacon("http://{{hosts[][nonexistent]}}", exceedPayload);
    assert_false(success, "calling 'navigator.sendBeacon()' with payload size exceeding the maximum size must fail");
}, "Verify calling 'navigator.sendBeacon()' with a large payload returns 'false'.");

test(function() {
    var invalidUrl = "http://invalid:url";
    assert_throws_js(TypeError, function() { navigator.sendBeacon(invalidUrl, smallPayload); },
        `calling 'navigator.sendBeacon()' with an invalid URL '${invalidUrl}' must throw a TypeError`);
}, "Verify calling 'navigator.sendBeacon()' with an invalid URL throws an exception.");

test(function() {
    var invalidUrl = "nothttp://invalid.url";
    assert_throws_js(TypeError, function() { navigator.sendBeacon(invalidUrl, smallPayload); },
         `calling 'navigator.sendBeacon()' with a non-http(s) URL '${invalidUrl}' must throw a TypeError`);
}, "Verify calling 'navigator.sendBeacon()' with a URL that is not a http(s) scheme throws an exception.");

// We'll validate that we can send one beacon that uses our entire Quota and then fail to send one that is just one char.
promise_test(async () => {
    function wait(ms) {
        return new Promise(res => step_timeout(res, ms));
    }
    const url = '/fetch/api/resources/trickle.py?count=1&ms=0';
    assert_true(navigator.sendBeacon(url, maxPayload),
                "calling 'navigator.sendBeacon()' with our max payload size should succeed.");

    // Now we'll send just one character.
    assert_false(navigator.sendBeacon(url, '1'),
                 "calling 'navigator.sendBeacon()' with just one char should fail while our Quota is used up.");

    for (let i = 0; i < 20; ++i) {
        await wait(100);
        if (navigator.sendBeacon(url, maxPayload)) {
           return;
        }
    }
    assert_unreached('The quota should recover after fetching.');
}, "Verify the behavior after the quota is exhausted.");

done();
