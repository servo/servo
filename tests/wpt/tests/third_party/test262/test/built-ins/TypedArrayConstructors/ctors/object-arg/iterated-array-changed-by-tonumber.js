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

testWithTypedArrayConstructors(function(TypedArray) {
  let values = [0, {
    valueOf() {
      // Removes all array elements. Caller must have saved all elements.
      values.length = 0;
      return 100;
    }
  }, 2];

  // Constructor called with array which uses the built-in array iterator.
  var ta = new TypedArray(values);

  assert.sameValue(ta.length, 3);
  assert.sameValue(ta[0], 0);
  assert.sameValue(ta[1], 100);
  assert.sameValue(ta[2], 2);
}, null, ["passthrough"]);
