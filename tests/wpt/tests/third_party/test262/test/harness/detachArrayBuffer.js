// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including detachArrayBuffer.js will expose a function:

        $DETACHBUFFER

    $DETACHBUFFER relies on the presence of a definition for $262.detachArrayBuffer.

    Without a definition, calling $DETACHBUFFER will result in a ReferenceError
---*/

var ab = new ArrayBuffer(1);
var threw = false;

try {
  $DETACHBUFFER(ab);
} catch(err) {
  threw = true;
  if (err.constructor !== ReferenceError) {
    throw new Error(
      'Expected a ReferenceError, but a "' + err.constructor.name +
      '" was thrown.'
    );
  }
}

if (threw === false) {
  throw new Error('Expected a ReferenceError, but no error was thrown.');
}


