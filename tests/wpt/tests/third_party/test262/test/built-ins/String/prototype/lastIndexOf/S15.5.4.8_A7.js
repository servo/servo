// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf can't be used as constructor
es5id: 15.5.4.8_A7
description: Checking if creating the String.prototype.lastIndexOf object fails
---*/

var FACTORY = String.prototype.lastIndexOf;

assert.throws(TypeError, function() {
  new FACTORY;
});
