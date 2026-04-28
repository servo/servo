// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
/*---
esid: sec-array.from
description: >
  If this is a constructor, and items doesn't have an @@iterator,
  returns a new instance of this
info: |
  22.1.2.1 Array.from ( items [ , mapfn [ , thisArg ] ] )

  4. Let usingIterator be GetMethod(items, @@iterator).
  ...
  6. If usingIterator is not undefined, then
  ...
  12. If IsConstructor(C) is true, then
    a. Let A be Construct(C, «len»).
  13. Else,
    a. Let A be ArrayCreate(len).
  ...
  19. Return A.
---*/

var result;

function MyCollection() {
  this.args = arguments;
}

result = Array.from.call(MyCollection, {
  length: 42
});

assert.sameValue(result.args.length, 1, 'The value of result.args.length is expected to be 1');
assert.sameValue(result.args[0], 42, 'The value of result.args[0] is expected to be 42');
assert(
  result instanceof MyCollection,
  'The result of evaluating (result instanceof MyCollection) is expected to be true'
);
