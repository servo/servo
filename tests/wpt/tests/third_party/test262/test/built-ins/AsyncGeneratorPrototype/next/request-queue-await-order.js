// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-next
description: next() requests from iterator processed in order, await
info: >
  AsyncGenerator.prototype.next ( value )
  1. Let generator be the this value.
  2. Let completion be NormalCompletion(value).
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  4. Let queue be generator.[[AsyncGeneratorQueue]].
  5. Let request be AsyncGeneratorRequest{[[Completion]]: completion,
     [[Capability]]: promiseCapability}.
  6. Append request to the end of queue.
  ...

  AsyncGeneratorResolve ( generator, value, done )
  ...
  2. Let queue be generator.[[AsyncGeneratorQueue]].
  3. Assert: queue is not an empty List.
  4. Remove the first element from queue and let next be the value of that element.
  ...

flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var yieldorder = 0;
var resolveLatePromise;

function resolveLater() {
  return new Promise(resolve => {
    resolveLatePromise = resolve;
  });
}

async function* g() {
  yield resolveLater();
  yield ++yieldorder;
}

var iter = g();

assert.sameValue(yieldorder, 0);

var item1 = iter.next();
var item2 = iter.next();
var item3 = iter.next();

async function awaitnexts() {
  assert.sameValue((await item3).value, undefined)
  assert.sameValue(yieldorder, 2, "All next requests have been proccessed.")
  assert.sameValue((await item2).value, 2)
  assert.sameValue((await item1).value, 1)
}

asyncTest(awaitnexts);

// At this point:
//   yieldorder == 0
//   item1 is an unresolved promise
resolveLatePromise(++yieldorder);
