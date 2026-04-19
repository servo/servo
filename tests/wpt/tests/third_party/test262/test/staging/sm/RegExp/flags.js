// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Implement RegExp.prototype.flags
info: bugzilla.mozilla.org/show_bug.cgi?id=1108467
esid: pending
---*/

assert.sameValue(RegExp.prototype.flags, "");
assert.sameValue(/foo/iymg.flags, "gimy");
assert.sameValue(RegExp("").flags, "");
assert.sameValue(RegExp("", "mygi").flags, "gimy");
assert.sameValue(RegExp("", "mygui").flags, "gimuy");
assert.sameValue(genericFlags({}), "");
assert.sameValue(genericFlags({ignoreCase: true}), "i");
assert.sameValue(genericFlags({sticky:1, unicode:1, global: 0}), "uy");
assert.sameValue(genericFlags({__proto__: {multiline: true}}), "m");
assert.sameValue(genericFlags(new Proxy({}, {get(){return true}})), "dgimsuvy");

assert.throws(TypeError, () => genericFlags());
assert.throws(TypeError, () => genericFlags(1));
assert.throws(TypeError, () => genericFlags(""));

function genericFlags(obj) {
    return Object.getOwnPropertyDescriptor(RegExp.prototype,"flags").get.call(obj);
}
