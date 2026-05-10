// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Boolean constructor has the property "prototype"
esid: sec-boolean.prototype
description: Checking existence of the property "prototype"
---*/
assert(Boolean.hasOwnProperty("prototype"), 'Boolean.hasOwnProperty("prototype") must return true');
