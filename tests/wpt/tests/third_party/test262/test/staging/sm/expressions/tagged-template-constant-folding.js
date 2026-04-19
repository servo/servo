/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Don't let constant-folding in the MemberExpression part of a tagged template cause an incorrect |this| be passed to the callee
info: bugzilla.mozilla.org/show_bug.cgi?id=1182373
esid: pending
---*/

var prop = "global";

var obj = { prop: "obj", f: function() { return this.prop; } };

assert.sameValue(obj.f``, "obj");
assert.sameValue((true ? obj.f : null)``, "global");
