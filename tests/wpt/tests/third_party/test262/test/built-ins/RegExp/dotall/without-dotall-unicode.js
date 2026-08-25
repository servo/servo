// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Without the dotAll flag, . does not match newlines in Unicode mode
info: |
  21.2.2.8 Atom
  The production Atom::. evaluates as follows:
    1. If DotAll is true, then
      a. Let A be the set of all characters.
    2. Otherwise, let A be the set of all characters except LineTerminator.
    3. Call CharacterSetMatcher(A, false) and return its Matcher result.

esid: sec-atom
features: [u180e]
---*/

// The behavior is the same regardless of the m flag
for (let re of [/^.$/u, /^.$/um]) {
  assert(re.test("a"));
  assert(re.test("3"));
  assert(re.test("π"));
  assert(re.test("\u2027"));
  assert(re.test("\u0085"));
  assert(re.test("\v"));
  assert(re.test("\f"));
  assert(re.test("\u180E"));
  assert(re.test("\u{10300}"), "Supplementary plane matched by a single .");
  assert(!re.test("\n"));
  assert(!re.test("\r"));
  assert(!re.test("\u2028"));
  assert(!re.test("\u2029"));
  assert(re.test("\uD800"));
  assert(re.test("\uDFFF"));
}
