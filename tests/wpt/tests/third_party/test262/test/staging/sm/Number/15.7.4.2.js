/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

assert.throws(TypeError, function() { Number.prototype.toString.call(true); });
assert.throws(TypeError, function() { Number.prototype.toString.call(""); });
assert.throws(TypeError, function() { Number.prototype.toString.call({}); });
assert.throws(TypeError, function() { Number.prototype.toString.call(null); });
assert.throws(TypeError, function() { Number.prototype.toString.call([]); });
assert.throws(TypeError, function() { Number.prototype.toString.call(undefined); });
assert.throws(TypeError, function() { Number.prototype.toString.call(new Boolean(true)); });

assert.sameValue(Number.prototype.toString.call(42), "42");
assert.sameValue(Number.prototype.toString.call(new Number(42)), "42");

function testAround(middle)
{
    var range = 260;
    var low = middle - range/2;
    for (var i = 0; i < range; ++i)
        assert.sameValue(low + i, parseInt(String(low + i)));
}

testAround(-Math.pow(2,32));
testAround(-Math.pow(2,16));
testAround(0);
testAround(+Math.pow(2,16));
testAround(+Math.pow(2,32));

