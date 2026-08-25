// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Make RegExp.prototype.toString to be a generic function.
info: bugzilla.mozilla.org/show_bug.cgi?id=1079919
esid: pending
---*/

assert.sameValue(RegExp.prototype.toString(), "/(?:)/");
assert.sameValue(/foo/.toString(), "/foo/");
assert.sameValue(/foo/i.toString(), "/foo/i");
assert.sameValue(/foo/gimy.toString(), "/foo/gimy");
assert.sameValue(/foo/igym.toString(), "/foo/gimy");
assert.sameValue(/\n\r/.toString(), "/\\n\\r/");
assert.sameValue(/\u2028\u2029/.toString(), "/\\u2028\\u2029/");
assert.sameValue(/\//.toString(), "/\\//");
assert.sameValue(RegExp().toString(), "/(?:)/");
assert.sameValue(RegExp("", "gimy").toString(), "/(?:)/gimy");
assert.sameValue(RegExp("\n\r").toString(), "/\\n\\r/");
assert.sameValue(RegExp("\u2028\u2029").toString(), "/\\u2028\\u2029/");
assert.sameValue(RegExp("/").toString(), "/\\//");

assert.throws(TypeError, () => RegExp.prototype.toString.call());
assert.throws(TypeError, () => RegExp.prototype.toString.call(1));
assert.throws(TypeError, () => RegExp.prototype.toString.call(""));
assert.sameValue(RegExp.prototype.toString.call({}), "/undefined/undefined");
assert.sameValue(RegExp.prototype.toString.call({ source:"foo", flags:"bar" }), "/foo/bar");

var a = [];
var p = new Proxy({}, {
  get(that, name) {
    a.push(name);
    return {
      toString() {
        a.push(name + "-tostring");
        return name;
      }
    };
  }
});
assert.sameValue(RegExp.prototype.toString.call(p), "/source/flags");
assert.sameValue(a.join(","), "source,source-tostring,flags,flags-tostring");
