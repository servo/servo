/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

var gnew = $262.createRealm().global;

gnew.eval("function f() { return this; }");
var f = gnew.f;
assert.sameValue(f(), gnew);

gnew.eval("function g() { 'use strict'; return this; }");
var g = gnew.g;
assert.sameValue(g(), undefined);

