// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Implement RegExp.prototype.{global, ignoreCase, multiline, sticky, unicode} - property descriptor
info: bugzilla.mozilla.org/show_bug.cgi?id=1120169
esid: pending
---*/

var getters = [
  "flags",
  "global",
  "ignoreCase",
  "multiline",
  "source",
  "sticky",
  "unicode",
];

for (var name of getters) {
  var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, name);
  assert.sameValue(desc.configurable, true);
  assert.sameValue(desc.enumerable, false);
  assert.sameValue("writable" in desc, false);
  assert.sameValue("get" in desc, true);
}
