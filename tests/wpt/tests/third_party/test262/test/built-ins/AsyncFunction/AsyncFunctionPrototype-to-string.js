// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-prototype-properties-toStringTag
description: >
  %AsyncFunctionPrototype% has a Symbol.toStringTag property of "AsyncFunction"
includes: [propertyHelper.js]
features: [Symbol.toStringTag]
---*/

var AsyncFunction = async function foo() {}.constructor;
var AFP = AsyncFunction.prototype;
assert.sameValue(AFP[Symbol.toStringTag], "AsyncFunction", "toStringTag value");
verifyNotWritable(AFP, Symbol.toStringTag);
verifyNotEnumerable(AFP, Symbol.toStringTag);
verifyConfigurable(AFP, Symbol.toStringTag);
