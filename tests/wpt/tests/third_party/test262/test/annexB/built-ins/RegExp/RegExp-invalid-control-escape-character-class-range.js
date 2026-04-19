// Copyright 2017 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-annexB-ClassAtomNoDash
description: >
  Invalid \c in a range behaves like [\\c-_]
info: |
  ClassAtomNoDash :: `\`

  The production ClassAtomNoDash :: `\` evaluates as follows:
    1. Return the CharSet containing the single character `\`.
---*/

let re = /[\\c-f]/

assert(re.test("\\"))
assert(!re.test("b"))
assert(re.test("c"))
assert(re.test("d"))
assert(re.test("e"))
assert(re.test("f"))
assert(!re.test("g"))
assert(!re.test("-"))
