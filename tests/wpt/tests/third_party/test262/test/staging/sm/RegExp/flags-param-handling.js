// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
assert.sameValue(RegExp(/foo/my).flags, "my");
assert.sameValue(RegExp(/foo/, "gi").flags, "gi");
assert.sameValue(RegExp(/foo/my, "gi").flags, "gi");
assert.sameValue(RegExp(/foo/my, "").flags, "");
assert.sameValue(RegExp(/foo/my, undefined).flags, "my");
assert.throws(SyntaxError, () => RegExp(/foo/my, null));
assert.throws(SyntaxError, () => RegExp(/foo/my, "foo"));

assert.sameValue(/a/.compile("b", "gi").flags, "gi");
assert.sameValue(/a/.compile(/b/my).flags, "my");
assert.sameValue(/a/.compile(/b/my, undefined).flags, "my");
assert.throws(TypeError, () => /a/.compile(/b/my, "gi"));
assert.throws(TypeError, () => /a/.compile(/b/my, ""));
assert.throws(TypeError, () => /a/.compile(/b/my, null));
assert.throws(TypeError, () => /a/.compile(/b/my, "foo"));

