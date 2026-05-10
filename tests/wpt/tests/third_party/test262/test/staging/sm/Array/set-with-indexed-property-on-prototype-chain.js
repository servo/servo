// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function ensureSetterCalledOnce(fn, value, index) {
    var setterCalled = false;
    Object.defineProperty(Array.prototype, index, {
        configurable: true,
        set: function(v) {
            assert.sameValue(setterCalled, false);
            setterCalled = true;
            assert.sameValue(v, value);
        }
    });

    assert.sameValue(setterCalled, false);
    fn();
    assert.sameValue(setterCalled, true);

    delete Array.prototype[index];
}

ensureSetterCalledOnce(function() {
    [].push("push");
}, "push", 0);

ensureSetterCalledOnce(function() {
    [/* hole */, "reverse"].reverse();
}, "reverse", 0);

ensureSetterCalledOnce(function() {
    ["reverse", /* hole */,].reverse();
}, "reverse", 1);

ensureSetterCalledOnce(function() {
    [/* hole */, "shift"].shift();
}, "shift", 0);

ensureSetterCalledOnce(function() {
    [/* hole */, "sort"].sort();
}, "sort", 0);

ensureSetterCalledOnce(function() {
    [/* hole */, undefined].sort();
}, undefined, 0);

ensureSetterCalledOnce(function() {
    [].splice(0, 0, "splice");
}, "splice", 0);

ensureSetterCalledOnce(function() {
    [/* hole */, "splice"].splice(0, 1);
}, "splice", 0);

ensureSetterCalledOnce(function(v) {
    ["splice", /* hole */,].splice(0, 0, "item");
}, "splice", 1);

ensureSetterCalledOnce(function() {
    [].unshift("unshift");
}, "unshift", 0);

