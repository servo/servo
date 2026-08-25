// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: sec-object.prototype.tostring
description: Object.prototype.toString has no prototype property
---*/

assert.sameValue(
  Object.prototype.toString.hasOwnProperty("prototype"),
  false,
  "Object.prototype.toString.hasOwnProperty(\"prototype\") returns false"
);
