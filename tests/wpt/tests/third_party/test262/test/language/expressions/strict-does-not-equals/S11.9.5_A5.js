// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are String-s.
    Return false, if x and y are exactly the same sequence of characters; otherwise, return true
es5id: 11.9.5_A5
description: x and y are primitive strings
---*/

//CHECK#1
if ("" !== "") {
  throw new Test262Error('#1: "" === ""');
}

//CHECK#2
if (" " !== " ") {
  throw new Test262Error('#2: " " === " "');
}

//CHECK#3
if ("string" !== "string") {
  throw new Test262Error('#3: "string" === "string"');
}

//CHECK#4
if (!(" string" !== "string ")) {
  throw new Test262Error('#4: " string" !== "string "');
}

//CHECK#5
if (!("1.0" !== "1")) {
  throw new Test262Error('#5: "1.0" !== "1"');
}
