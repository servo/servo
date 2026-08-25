// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.normalize error when normalization form parameter is not an atom
info: bugzilla.mozilla.org/show_bug.cgi?id=1145326
esid: pending
---*/

function test() {
  assert.sameValue("abc".normalize("NFKC".split("").join("")), "abc");
  assert.sameValue("abc".normalize("NFKCabc".replace("abc", "")), "abc");
  assert.sameValue("abc".normalize("N" + "F" + "K" + "C"), "abc");
}

if ("normalize" in String.prototype) {
  // String.prototype.normalize is not enabled in all builds.
  test();
}
