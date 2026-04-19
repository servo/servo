/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Test courtesy of Olov Lassus <olov.lassus@gmail.com>.

function keys(o) {
    var a = [];
    for (var key in o) {
        a.push(key);
    }
    return a;
}

var obj = {
    'a': function() {}, 'b': function() {}, 'c': function() {}
};
var orig_order = keys(obj).toString();
var tmp = obj["b"];
var read_order = keys(obj).toString();

assert.sameValue(orig_order, read_order,
              "property enumeration order should not change after reading a method value");
