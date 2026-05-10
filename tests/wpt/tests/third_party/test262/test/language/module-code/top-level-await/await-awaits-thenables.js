// Copyright 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AwaitExpression
description: >
  Await can await any thenable.
flags: [module, async]
features: [top-level-await]
---*/

var thenable = {
  then: function (resolve, reject) {
    resolve(42);
  }
}

assert.sameValue(await thenable, 42);

$DONE();
