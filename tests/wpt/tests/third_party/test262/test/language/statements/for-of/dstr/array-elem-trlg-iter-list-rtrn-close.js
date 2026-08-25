// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-list-rtrn-close.case
// - src/dstr-assignment/default/for-of.template
/*---
description: IteratorClose is invoked when evaluation of AssignmentElementList returns a "return" completion and the iterator has not been marked as "done" (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol.iterator, generators, destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
      for ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]

    ArrayAssignmentPattern :
        [ AssignmentElementList , Elisionopt AssignmentRestElementopt ]

    [...]
    3. Let iteratorRecord be Record {[[iterator]]: iterator, [[done]]: false}.
    4. Let status be the result of performing
       IteratorDestructuringAssignmentEvaluation of AssignmentElementList using
       iteratorRecord as the argument.
    5. If status is an abrupt completion, then
       a. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
          status).
       b. Return Completion(status).

    7.4.6 IteratorClose( iterator, completion )

    [...]
    6. Let innerResult be Call(return, iterator, « »).
    7. If completion.[[type]] is throw, return Completion(completion).
    8. If innerResult.[[type]] is throw, return Completion(innerResult).

---*/
var nextCount = 0;
var returnCount = 0;
var unreachable = 0;
var thisValue = null;
var args = null;
var iterable = {};
var iterator = {
  next: function() {
    nextCount += 1;
    return {done: false, value: undefined};
  },
  return: function() {
    returnCount += 1;
    thisValue = this;
    args = arguments;
    return {};
  }
};
var iter, result;

iterable[Symbol.iterator] = function() {
  return iterator;
};

function* g() {

var counter = 0;

for ([ {} = yield , ] of [iterable]) {
  unreachable += 1;
  counter += 1;
}

assert.sameValue(counter, 1);

};

iter = g();
iter.next();

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 0);

result = iter.return(888);

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 1);
assert.sameValue(unreachable, 0, 'Unreachable statement was not executed');
assert.sameValue(result.value, 888);
assert(result.done, 'Iterator correctly closed');
assert.sameValue(thisValue, iterator, 'correct `this` value');
assert(!!args, 'arguments object provided');
assert.sameValue(args.length, 0, 'zero arguments specified');
