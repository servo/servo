// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure that we don't misorder subclassing accesses with respect to
// accessing regex arg internal slots
//
// Test credit André Bargull.

var re = /a/;
var newRe = Reflect.construct(RegExp, [re], Object.defineProperty(function(){}.bind(null), "prototype", {
  get() {
    re.compile("b");
    return RegExp.prototype;
  }
}));
assert.sameValue(newRe.source, "a");

