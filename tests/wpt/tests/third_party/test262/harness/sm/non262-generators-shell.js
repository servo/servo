/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*---
defines: [assertIteratorResult, assertIteratorNext, assertIteratorDone]
---*/

function assertIteratorResult(result, value, done) {
    assert.sameValue(result.value, value);
    assert.sameValue(result.done, done);
}
function assertIteratorNext(iter, value) {
    assertIteratorResult(iter.next(), value, false);
}
function assertIteratorDone(iter, value) {
    assertIteratorResult(iter.next(), value, true);
}
