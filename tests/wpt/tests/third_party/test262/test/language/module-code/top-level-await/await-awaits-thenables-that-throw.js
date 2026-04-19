// Copyright 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AwaitExpression
description: >
  Await can await any thenable.
flags: [module, async]
features: [top-level-await]
---*/

var error = {};
var thenable = {
  then: function (resolve, reject) {
    throw error;
  }
}

var caught = false;
try {
  await thenable;
} catch(e) {
  caught = e;
  
}

assert.sameValue(caught, error);

$DONE();
