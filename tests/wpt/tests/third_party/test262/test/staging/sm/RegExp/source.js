// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Implement RegExp.prototype.source
info: bugzilla.mozilla.org/show_bug.cgi?id=1120169
esid: pending
---*/

assert.sameValue(RegExp.prototype.source, "(?:)");
assert.sameValue(/foo/.source, "foo");
assert.sameValue(/foo/iymg.source, "foo");
assert.sameValue(/\//.source, "\\/");
assert.sameValue(/\n\r/.source, "\\n\\r");
assert.sameValue(/\u2028\u2029/.source, "\\u2028\\u2029");
assert.sameValue(RegExp("").source, "(?:)");
assert.sameValue(RegExp("", "mygi").source, "(?:)");
assert.sameValue(RegExp("/").source, "\\/");
assert.sameValue(RegExp("\n\r").source, "\\n\\r");
assert.sameValue(RegExp("\u2028\u2029").source, "\\u2028\\u2029");

assert.throws(TypeError, () => genericSource());
assert.throws(TypeError, () => genericSource(1));
assert.throws(TypeError, () => genericSource(""));
assert.throws(TypeError, () => genericSource({}));
assert.throws(TypeError, () => genericSource(new Proxy(/foo/, {get(){ return true; }})));

function genericSource(obj) {
    return Object.getOwnPropertyDescriptor(RegExp.prototype, "source").get.call(obj);
}
