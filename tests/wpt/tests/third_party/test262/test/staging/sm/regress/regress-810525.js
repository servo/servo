/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

assert.throws(TypeError, () => eval("({p:1, q:2}).m()"));
assert.throws(TypeError, () => eval("[].m()"));
assert.throws(TypeError, () => eval("[1,2,3].m()"));
assert.throws(TypeError, () => eval("/hi/.m()"));

