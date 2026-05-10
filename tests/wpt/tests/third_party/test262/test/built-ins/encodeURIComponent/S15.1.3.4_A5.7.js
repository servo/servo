// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The encodeURIComponent property can't be used as constructor
esid: sec-encodeuricomponent-uricomponent
description: >
    If property does not implement the internal [[Construct]] method,
    throw a TypeError exception
---*/

//CHECK#1

try {
  new encodeURIComponent();
  throw new Test262Error('#1.1: new encodeURIComponent() throw TypeError. Actual: ' + (new encodeURIComponent()));
} catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: new encodeURIComponent() throw TypeError. Actual: ' + (e));
  }
}
