// Copyright (C) 2020 Salesforce.com. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-try-statement-runtime-semantics-evaluation
description: >
  Returns the correct completion values of try-catch-finally(Abrupt) in functions
info: |
  TryStatement : try Block Catch Finally

    Let B be the result of evaluating Block.
    If B.[[Type]] is throw, let C be CatchClauseEvaluation of Catch with argument B.[[Value]].
    Else, let C be B.
    Let F be the result of evaluating Finally.
    If F.[[Type]] is normal, set F to C.
    Return Completion(UpdateEmpty(F, undefined)).
---*/

var fn, count = {};

// 1: try Abrupt, catch Abrupt, finally Abrupt; Completion: finally
count.catch = 0;
count.finally = 0;
fn = function() {
  try {
    throw 'try';
  } catch(e) {
    count.catch += 1;
    throw 'catch';
  } finally {
    count.finally += 1;
    throw new Test262Error('finally'); // If F.[[Type]] is normal, set F to C.
  }
  return 'wat';
};

assert.throws(Test262Error, fn, '1: try Abrupt, catch Abrupt, finally Abrupt; Completion: finally');
assert.sameValue(count.catch, 1, '1: catch count');
assert.sameValue(count.finally, 1, '1: finally count');

// 2: try Abrupt, catch Return, finally Abrupt; Completion: finally
count.catch = 0;
count.finally = 0;
fn = function() {
  try {
    throw 'try';
  } catch(e) {
    count.catch += 1;
    return 'catch';
  } finally {
    count.finally += 1;
    throw new Test262Error('finally'); // If F.[[Type]] is normal, set F to C.
  }
  return 'wat';
};

assert.throws(Test262Error, fn, '2: try Abrupt, catch Return, finally Abrupt; Completion: finally');
assert.sameValue(count.catch, 1, '2: catch count');
assert.sameValue(count.finally, 1, '2: fiinally count');

// 3: try Return, catch Return, finally Abrupt; Completion: finally
count.catch = 0;
count.finally = 0;
fn = function() {
  try {
    return 'try';
  } catch(e) {
    count.catch += 1;
    return 'catch';
  } finally {
    count.finally += 1;
    throw new Test262Error('finally'); // If F.[[Type]] is normal, set F to C.
  }
  return 'wat';
};

assert.throws(Test262Error, fn, '3: try Normal, catch Normal, finally Abrupt; Completion: finally');
assert.sameValue(count.catch, 0, '3: catch count');
assert.sameValue(count.finally, 1, '3: fiinally count');
