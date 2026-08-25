/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
description: |
  Array.prototype.fill
info: bugzilla.mozilla.org/show_bug.cgi?id=911147
esid: pending
---*/

assert.sameValue(typeof [].fill, 'function');
assert.sameValue([].fill.length, 1);

// Default values for arguments and absolute values for negative start and end
// arguments are resolved correctly.
assert.deepEqual([].fill(1), []);
assert.deepEqual([1,1,1].fill(2), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, 1), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, 1, 2), [1,2,1]);
assert.deepEqual([1,1,1].fill(2, -2), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, -2, -1), [1,2,1]);
assert.deepEqual([1,1,1].fill(2, undefined), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, undefined, undefined), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, 1, undefined), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, undefined, 1), [2,1,1]);
assert.deepEqual([1,1,1].fill(2, 2, 1), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, -1, 1), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, -2, 1), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, 1, -2), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, 0.1), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, 0.9), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, 1.1), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, 0.1, 0.9), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, 0.1, 1.9), [2,1,1]);
assert.deepEqual([1,1,1].fill(2, 0.1, 1.9), [2,1,1]);
assert.deepEqual([1,1,1].fill(2, -0), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, 0, -0), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, NaN), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, 0, NaN), [1,1,1]);
assert.deepEqual([1,1,1].fill(2, false), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, true), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, "0"), [2,2,2]);
assert.deepEqual([1,1,1].fill(2, "1"), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, "-2"), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, "-2", "-1"), [1,2,1]);
assert.deepEqual([1,1,1].fill(2, {valueOf: ()=>1}), [1,2,2]);
assert.deepEqual([1,1,1].fill(2, 0, {valueOf: ()=>1}), [2,1,1]);

// fill works generically for objects, too.
assert.deepEqual([].fill.call({length: 2}, 2), {0: 2, 1: 2, length: 2});

var setterCalled = false;
var objWithSetter = {set "0"(val) { setterCalled = true}, length: 1};
[].fill.call(objWithSetter, 2);
assert.sameValue(setterCalled, true);

var setHandlerCallCount = 0;
var proxy = new Proxy({length: 3}, {set(t, i, v, r) { setHandlerCallCount++; return true; }});
[].fill.call(proxy, 2);
assert.sameValue(setHandlerCallCount, 3);

var valueOfCallCount = 0;
var typedArray = new Uint8ClampedArray(3);
[].fill.call(typedArray, {valueOf: function() {valueOfCallCount++; return 2000;}});
assert.sameValue(valueOfCallCount, 3);
assert.sameValue(typedArray[0], 0xff);

// All remaining cases should throw.
var objWithGetterOnly = {get "0"() {return 1;}, length: 1};

var objWithReadOnlyProp = {length: 1};
Object.defineProperty(objWithReadOnlyProp, 0, {value: 1, writable: false});

var objWithNonconfigurableProp = {length: 1};
Object.defineProperty(objWithNonconfigurableProp, 0, {value: 1, configurable: false});

var frozenObj = {length: 1};
Object.freeze(frozenObj);

var frozenArray = [1, 1, 1];
Object.freeze(frozenArray);

assert.throws(TypeError, () => [].fill.call(objWithGetterOnly, 2));
assert.throws(TypeError, () => [].fill.call(objWithReadOnlyProp, 2));
assert.throws(TypeError, () => [].fill.call(objWithNonconfigurableProp, 2));
assert.throws(TypeError, () => [].fill.call(frozenObj, 2));
assert.throws(TypeError, () => [].fill.call(frozenArray, 2));
assert.throws(TypeError, () => [].fill.call("111", 2));
assert.throws(TypeError, () => [].fill.call(null, 2));
assert.throws(TypeError, () => [].fill.call(undefined, 2));
