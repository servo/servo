// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Let reservedURISet be a string containing one instance of each character valid
    in uriReserved plus "#"
esid: sec-decodeuri-encodeduri
description: Complex test
---*/

//CHECK#1
if (decodeURI("%3B%2F%3F%3A%40%26%3D%2B%24%2C%23") !== "%3B%2F%3F%3A%40%26%3D%2B%24%2C%23") {
  throw new Test262Error('#1: decodeURI("%3B%2F%3F%3A%40%26%3D%2B%24%2C%23") equal "%3B%2F%3F%3A%40%26%3D%2B%24%2C%23", not ";/?:@&=+$,#"');
}

//CHECK#2
if (decodeURI("%3b%2f%3f%3a%40%26%3d%2b%24%2c%23") !== "%3b%2f%3f%3a%40%26%3d%2b%24%2c%23") {
  throw new Test262Error('#2: decodeURI("%3b%2f%3f%3a%40%26%3d%2b%24%2c%23") equal "%3b%2f%3f%3a%40%26%3d%2b%24%2c%23", not ";/?:@&=+$,#" or "%3B%2F%3F%3A%40%26%3D%2B%24%2C%23"');
}
