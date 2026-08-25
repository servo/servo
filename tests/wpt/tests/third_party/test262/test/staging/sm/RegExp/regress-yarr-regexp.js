// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
var gcgcz = /((?:.)+)((?:.)*)/; /* Greedy capture, greedy capture zero. */
assert.compareArray(["a", "a", ""], gcgcz.exec("a"));
assert.compareArray(["ab", "ab", ""], gcgcz.exec("ab"));
assert.compareArray(["abc", "abc", ""], gcgcz.exec("abc"));

assert.compareArray(["a", ""], /((?:)*?)a/.exec("a"));
assert.compareArray(["a", ""], /((?:.)*?)a/.exec("a"));
assert.compareArray(["a", ""], /a((?:.)*)/.exec("a"));

assert.compareArray(["B", "B"], /([A-Z])/.exec("fooBar"));

// These just mustn't crash. See bug 872971
try { assert.sameValue(/x{2147483648}x/.test('1'), false); } catch (e) {}
try { assert.sameValue(/x{2147483648,}x/.test('1'), false); } catch (e) {}
try { assert.sameValue(/x{2147483647,2147483648}x/.test('1'), false); } catch (e) {}
// Same for these. See bug 813366
try { assert.sameValue("".match(/.{2147483647}11/), null); } catch (e) {}
try { assert.sameValue("".match(/(?:(?=g)).{2147483648,}/ + ""), null); } catch (e) {}
