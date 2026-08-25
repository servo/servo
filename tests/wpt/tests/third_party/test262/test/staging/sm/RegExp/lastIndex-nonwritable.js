// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Regexp.prototype.test/exec shouldn't change lastIndex if not writable.
info: bugzilla.mozilla.org/show_bug.cgi?id=1168416
esid: pending
---*/

var regex = /0/g;
Object.freeze(regex);
var str = "abc000";

var desc = Object.getOwnPropertyDescriptor(regex, "lastIndex");
assert.sameValue(desc.writable, false);
assert.sameValue(desc.value, 0);

assert.throws(TypeError, () => regex.test(str));

desc = Object.getOwnPropertyDescriptor(regex, "lastIndex");
assert.sameValue(desc.writable, false);
assert.sameValue(desc.value, 0);

assert.throws(TypeError, () => regex.exec(str));

desc = Object.getOwnPropertyDescriptor(regex, "lastIndex");
assert.sameValue(desc.writable, false);
assert.sameValue(desc.value, 0);
