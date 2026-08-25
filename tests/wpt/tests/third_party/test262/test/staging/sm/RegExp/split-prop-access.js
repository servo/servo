// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.split should call ToUint32(limit) before ToString(separator).
info: bugzilla.mozilla.org/show_bug.cgi?id=1287525
esid: pending
---*/

var accessed = false;

var rx = /a/;
Object.defineProperty(rx, Symbol.match, {
  get() {
    accessed = true;
  }
});
rx[Symbol.split]("abba");

assert.sameValue(accessed, true);
