// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-generators-shell.js]
description: |
  pending
esid: pending
---*/
// yield* calls @@iterator on the iterable to produce the iterator.

var log = '';

function IteratorWrapper(iterator) {
    return {
        next: function (val) {
            log += 'n';
            return iterator.next(val);
        },

        throw: function (exn) {
            log += 't';
            return iterator.throw(exn);
        }
    };
}

function IterableWrapper(iterable) {
    var ret = {};

    ret[Symbol.iterator] = function () {
        log += 'i';
        return IteratorWrapper(iterable[Symbol.iterator]());
    }

    return ret;
}

function* delegate(iter) { return yield* iter; }

var iter = delegate(IterableWrapper([1, 2, 3]));
assertIteratorNext(iter, 1);
assertIteratorNext(iter, 2);
assertIteratorNext(iter, 3);
assertIteratorDone(iter, undefined);

assert.sameValue(log, 'innnn');

iter = delegate([1, 2, 3]);
assertIteratorNext(iter, 1);
assertIteratorNext(iter, 2);
assertIteratorNext(iter, 3);
assertIteratorDone(iter, undefined);

assert.sameValue(log, 'innnn');

