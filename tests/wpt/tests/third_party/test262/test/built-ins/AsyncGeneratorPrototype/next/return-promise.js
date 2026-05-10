// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-next
description: >
  "next" returns a promise for an IteratorResult object
info: |
  AsyncGenerator.prototype.next ( value )
  1. Let generator be the this value.
  2. Let completion be NormalCompletion(value).
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  9. Return promiseCapability.[[Promise]].

features: [async-iteration]
---*/

async function* g() {}
var result = g().next()

assert(result instanceof Promise)
