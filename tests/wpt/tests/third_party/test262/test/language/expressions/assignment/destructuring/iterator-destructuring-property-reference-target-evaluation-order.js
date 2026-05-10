// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-iteratordestructuringassignmentevaluation
description: >
    Ensure correct evaluation order when destructuring target is property reference.
info: |
    12.15.5.3 Runtime Semantics: IteratorDestructuringAssignmentEvaluation

    AssignmentElement : DestructuringAssignmentTarget Initializer

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
        a. Let lref be the result of evaluating DestructuringAssignmentTarget.
        b. ReturnIfAbrupt(lref).
    2. If iteratorRecord.[[Done]] is false, then
        a. Let next be IteratorStep(iteratorRecord.[[Iterator]]).
        ...
    3. If iteratorRecord.[[Done]] is true, let value be undefined.
    ...
    5. Else, let v be value.
    ...
    8. Return ? PutValue(lref, v).
features: [Symbol.iterator]
includes: [compareArray.js]
---*/


var log = [];

function source() {
    log.push("source");
    var iterator = {
        next: function() {
            log.push("iterator-step");
            return {
                get done() {
                    log.push("iterator-done");
                    return true;
                },
                get value() {
                    // Note: This getter shouldn't be called.
                    log.push("iterator-value");
                }
            };
        }
    };
    var source = {};
    source[Symbol.iterator] = function() {
        log.push("iterator");
        return iterator;
    };
    return source;
}
function target() {
    log.push("target");
    return target = {
        set q(v) {
            log.push("set");
        }
    };
}
function targetKey() {
    log.push("target-key");
    return {
        toString: function() {
            log.push("target-key-tostring");
            return "q";
        }
    };
}

([target()[targetKey()]] = source());

assert.compareArray(log, [
    "source", "iterator",
    "target", "target-key",
    "iterator-step", "iterator-done",
    "target-key-tostring", "set",
]);
