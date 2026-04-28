// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray
description: >
  Modifications to input array after iteration are handled correctly.
info: |
  TypedArray ( ...args )

  ...
  6. Else,
    ...
    b. If firstArgument is an Object, then
      ...
      iv. Else,
        ...
        2. Let usingIterator be ? GetMethod(firstArgument, @@iterator).
        3. If usingIterator is not undefined, then
          a. Let values be ? IteratorToList(? GetIteratorFromMethod(firstArgument, usingIterator)).
          b. Perform ? InitializeTypedArrayFromList(O, values).
        ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

let ArrayIteratorPrototype = Object.getPrototypeOf([].values());
let values;

// Modify the built-in ArrayIteratorPrototype `next` method.
ArrayIteratorPrototype.next = function() {
  let done = values.length === 0;
  let value = values.pop();
  return {value, done};
};

testWithTypedArrayConstructors(function(TypedArray) {
  // Reset `values` array.
  values = [1, 2, 3, 4];

  // Constructor called with array which uses the modified array iterator.
  var ta = new TypedArray([0]);

  assert.sameValue(ta.length, 4);
  assert.sameValue(ta[0], 4);
  assert.sameValue(ta[1], 3);
  assert.sameValue(ta[2], 2);
  assert.sameValue(ta[3], 1);
}, null, ["passthrough"]);
