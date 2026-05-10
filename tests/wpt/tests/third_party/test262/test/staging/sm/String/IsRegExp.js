// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.{startsWith,endsWith,includes} should call IsRegExp.
info: bugzilla.mozilla.org/show_bug.cgi?id=1054755
esid: pending
---*/

for (var method of ["startsWith", "endsWith", "includes"]) {
  for (var re of [/foo/, new RegExp()]) {
    assert.throws(TypeError, () => "foo"[method](re));

    re[Symbol.match] = false;
    "foo"[method](re);
  }

  for (var v1 of [true, 1, "bar", [], {}, Symbol.iterator]) {
    assert.throws(TypeError, () => "foo"[method]({ [Symbol.match]: v1 }));
  }

  for (var v2 of [false, 0, undefined, ""]) {
    "foo"[method]({ [Symbol.match]: v2 });
  }
}
