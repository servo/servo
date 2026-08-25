// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure that we don't ToString the second argument until /after/ doing
// the appropriate subclassing lookups

var didLookup = false;

var re = /a/;
var flags = { toString() { assert.sameValue(didLookup, true); return "g"; } };
var newRe = Reflect.construct(RegExp, [re, flags],
                              Object.defineProperty(function(){}.bind(null), "prototype", {
  get() {
    didLookup = true;
    return RegExp.prototype;
  }
}));

assert.sameValue(Object.getPrototypeOf(newRe), RegExp.prototype);
assert.sameValue(didLookup, true);


