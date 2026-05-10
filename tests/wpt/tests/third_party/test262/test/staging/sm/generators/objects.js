// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

// This file was written by Andy Wingo <wingo@igalia.com> and originally
// contributed to V8 as generators-objects.js, available here:
//
// http://code.google.com/p/v8/source/browse/branches/bleeding_edge/test/mjsunit/harmony/generators-objects.js

// Test aspects of the generator runtime.

// Test the properties and prototype of a generator object.
function TestGeneratorObject() {
  function* g() { yield 1; }

  var iter = g();
  assert.sameValue(Object.getPrototypeOf(iter), g.prototype);
  assert.sameValue(iter instanceof g, true);
  assert.sameValue(String(iter), "[object Generator]");
  assert.compareArray(Object.getOwnPropertyNames(iter), []);
  assert.notSameValue(g(), iter);
}
TestGeneratorObject();


// Test the methods of generator objects.
function TestGeneratorObjectMethods() {
  function* g() { yield 1; }
  var iter = g();

  assert.sameValue(iter.next.length, 1);
  assert.sameValue(iter.return.length, 1);
  assert.sameValue(iter.throw.length, 1);

  function TestNonGenerator(non_generator) {
    assert.throws(TypeError, function() { iter.next.call(non_generator); });
    assert.throws(TypeError, function() { iter.next.call(non_generator, 1); });
    assert.throws(TypeError, function() { iter.return.call(non_generator, 1); });
    assert.throws(TypeError, function() { iter.throw.call(non_generator, 1); });
    assert.throws(TypeError, function() { iter.close.call(non_generator); });
  }

  TestNonGenerator(1);
  TestNonGenerator({});
  TestNonGenerator(function(){});
  TestNonGenerator(g);
  TestNonGenerator(g.prototype);
}
TestGeneratorObjectMethods();


