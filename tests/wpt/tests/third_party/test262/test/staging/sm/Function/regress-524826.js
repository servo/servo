/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 524826;
var summary = 'null-closure property initialiser mis-brands object literal scope';
var actual;
var expect;

function make(g) {
    var o = {f: function(a,b) { return a*b; }, g: g};
    return o;
}
var z = -1;
var x = make(function(c) { return c*z; });
var y = make(function(c) { return -c*z; });

function callg(o, c) { return o.g(c); };
actual = callg(x, 1);
expect = -callg(y, 1);

assert.sameValue(expect, actual, summary);
