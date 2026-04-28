// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/

function checkResult(actual, expected)
{
  assert.sameValue(actual.value, expected.value);
  assert.sameValue(actual.done, expected.done);
}

var iteratorFunction = Uint8Array.prototype[Symbol.iterator];

var thisGlobal = this;
var otherGlobal = $262.createRealm().global;

var thisNext = new Uint8Array()[Symbol.iterator]().next

for (const constructor of typedArrayConstructors)
{
  assert.sameValue(new constructor()[Symbol.iterator]().next, thisNext);

  var globals =
    [
     [thisGlobal, thisGlobal],
     [thisGlobal, otherGlobal],
     [otherGlobal, otherGlobal],
     [otherGlobal, thisGlobal],
    ];

  for (const [arrayGlobal, bufferGlobal] of globals)
  {
    var arr, buffer, iterator;

    function arrayBufferIterator()
    {
      var byteLength = 2 * constructor.BYTES_PER_ELEMENT;
      var buf = new bufferGlobal.ArrayBuffer(byteLength);
      var tarray = new arrayGlobal[constructor.name](buf);

      tarray[0] = 1;
      tarray[1] = 2;

      return [tarray, buf, Reflect.apply(iteratorFunction, tarray, [])];
    }

    [arr, buffer, iterator] = arrayBufferIterator();
    checkResult(thisNext.call(iterator), {value: 1, done: false});
    checkResult(thisNext.call(iterator), {value: 2, done: false});
    checkResult(thisNext.call(iterator), {value: undefined, done: true});

    // Test an exhausted iterator.
    bufferGlobal.$262.detachArrayBuffer(buffer);
    checkResult(thisNext.call(iterator), {value: undefined, done: true});

    // Test an all-but-exhausted iterator.
    [arr, buffer, iterator] = arrayBufferIterator();
    checkResult(thisNext.call(iterator), {value: 1, done: false});
    checkResult(thisNext.call(iterator), {value: 2, done: false});

    bufferGlobal.$262.detachArrayBuffer(buffer);
    assert.throws(TypeError, () => thisNext.call(iterator));

    // Test an unexhausted iterator.
    [arr, buffer, iterator] = arrayBufferIterator();
    checkResult(thisNext.call(iterator), {value: 1, done: false});

    bufferGlobal.$262.detachArrayBuffer(buffer);
    assert.throws(TypeError, () => thisNext.call(iterator));
  }
}
