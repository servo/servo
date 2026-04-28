// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including detachArrayBuffer.js will expose a function:

        $DETACHBUFFER

    $DETACHBUFFER relies on the presence of a host definition for $262.detachArrayBuffer

includes: [detachArrayBuffer.js]
---*/

var $262 = {
  detachArrayBuffer() {
    throw new Test262Error('$262.detachArrayBuffer called.');
  },
  destroy() {}
};

var ab = new ArrayBuffer(1);
var threw = false;

try {
  $DETACHBUFFER(ab);
} catch(err) {
  threw = true;
  if (err.constructor !== Test262Error) {
    throw new Error(
      'Expected a Test262Error, but a "' + err.constructor.name +
      '" was thrown.'
    );
  }
  if (err.message !== '$262.detachArrayBuffer called.') {
    throw new Error(`Expected error message: ${err.message}`);
  }
}

if (threw === false) {
  throw new Error('Expected a Test262Error, but no error was thrown.');
}


