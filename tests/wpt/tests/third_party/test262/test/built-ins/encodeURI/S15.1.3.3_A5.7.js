// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The encodeURI property can't be used as constructor
esid: sec-encodeuri-uri
description: >
    If property does not implement the internal [[Construct]] method,
    throw a TypeError exception
---*/

//CHECK#1

try {
  new encodeURI();
  throw new Test262Error('#1.1: new encodeURI() throw TypeError. Actual: ' + (new encodeURI()));
} catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: new encodeURI() throw TypeError. Actual: ' + (e));
  }
}
