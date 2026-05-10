/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 533254;
var summary = 'init-method late in table-big initialiser screwup';

function f() {
    var proto = {p8:8};
    var obj = {
        p0:0, p1:1, p2:2, p3:3, p4:4, p5:5, p6:6, p7:7, p8:8, p9:9, 
        p10:0, p11:1, p12:2, p13:3, p14:4, p15:5, p16:6, p17:7, p18:8, p19:9, 
        m: function() { return 42; }
    };
    return obj;
}
var expect = f(),
    actual = f();

expect += '';
actual += '';
assert.sameValue(expect, actual, summary);
