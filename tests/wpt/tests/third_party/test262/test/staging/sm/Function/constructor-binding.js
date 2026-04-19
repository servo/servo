// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  A function created by Function constructor shouldn't have anonymous binding
info: bugzilla.mozilla.org/show_bug.cgi?id=636635
esid: pending
---*/

assert.sameValue(new Function("return typeof anonymous")(), "undefined");
assert.sameValue(new Function("return function() { return typeof anonymous; }")()(), "undefined");
assert.sameValue(new Function("return function() { eval(''); return typeof anonymous; }")()(), "undefined");
