// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.split should call ToUint32(limit) before ToString(separator).
info: bugzilla.mozilla.org/show_bug.cgi?id=1287521
esid: pending
---*/

var log = [];
"abba".split({
  toString() {
    log.push("separator-tostring");
    return "b";
  }
}, {
  valueOf() {
    log.push("limit-valueOf");
    return 0;
  }
});

assert.sameValue(log.join(","), "limit-valueOf,separator-tostring");
