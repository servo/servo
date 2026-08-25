// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Call replaceValue passing undefined as the this value
es5id: 15.5.4.11_A12
description: replaceValue tests that its this value is undefined
flags: [noStrict]
---*/

var global = this;
'x'.replace(/x/, function() {
  "use strict";

  if (this === global) {
    throw new Test262Error('#1: String replace leaks global');
  }
  if (this !== undefined) {
    throw new Test262Error('#2: replaceValue should be called with this===undefined. ' +
      'Actual: ' + this);
  }
  return 'y';
});
