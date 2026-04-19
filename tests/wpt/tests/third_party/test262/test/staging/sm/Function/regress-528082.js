/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 528082;
var summary = 'named function expression function-name-as-upvar slot botch';

function f() {
    return function g(a) { return function () { return g; }(); }();
}
var actual = typeof f();
var expect = "function";

assert.sameValue(expect, actual, summary);
