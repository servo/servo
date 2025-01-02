/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Error scope validation tests.

Note these must create their own device, not use GPUTest (that one already has error scopes on it).

TODO: (POSTV1) Test error scopes of different threads and make sure they go to the right place.
TODO: (POSTV1) Test that unhandled errors go the right device, and nowhere if the device was dropped.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { kErrorScopeFilters, kGeneratableErrorScopeFilters } from '../../capability_info.js';
import { ErrorTest } from '../../error_test.js';

export const g = makeTestGroup(ErrorTest);

g.test('simple').
desc(
  `
Tests that error scopes catches their expected errors, firing an uncaptured error event otherwise.

- Same error and error filter (popErrorScope should return the error)
- Different error from filter (uncaptured error should result)
    `
).
params((u) =>
u.combine('errorType', kGeneratableErrorScopeFilters).combine('errorFilter', kErrorScopeFilters)
).
fn(async (t) => {
  const { errorType, errorFilter } = t.params;
  t.device.pushErrorScope(errorFilter);

  if (errorType !== errorFilter) {
    // Different error case
    const uncapturedErrorEvent = await t.expectUncapturedError(() => {
      t.generateError(errorType);
    });
    t.expect(t.isInstanceOfError(errorType, uncapturedErrorEvent.error));

    const error = await t.device.popErrorScope();
    t.expect(error === null);
  } else {
    // Same error as filter
    t.generateError(errorType);
    const error = await t.device.popErrorScope();
    t.expect(t.isInstanceOfError(errorType, error));
  }
});

g.test('empty').
desc(
  `
Tests that popping an empty error scope stack should reject.
    `
).
fn((t) => {
  const promise = t.device.popErrorScope();
  t.shouldReject('OperationError', promise, { allowMissingStack: true });
});

g.test('parent_scope').
desc(
  `
Tests that an error bubbles to the correct parent scope.

- Different error types as the parent scope
- Different depths of non-capturing filters for the generated error
    `
).
params((u) =>
u.
combine('errorFilter', kGeneratableErrorScopeFilters).
combine('stackDepth', [1, 10, 100, 1000])
).
fn(async (t) => {
  const { errorFilter, stackDepth } = t.params;
  t.device.pushErrorScope(errorFilter);

  // Push a bunch of error filters onto the stack (none that match errorFilter)
  const unmatchedFilters = kErrorScopeFilters.filter((filter) => {
    return filter !== errorFilter;
  });
  for (let i = 0; i < stackDepth; i++) {
    t.device.pushErrorScope(unmatchedFilters[i % unmatchedFilters.length]);
  }

  // Cause the error and then pop all the unrelated filters.
  t.generateError(errorFilter);
  const promises = [];
  for (let i = 0; i < stackDepth; i++) {
    promises.push(t.device.popErrorScope());
  }
  const errors = await Promise.all(promises);
  t.expect(errors.every((e) => e === null));

  // Finally the actual error should have been caught by the parent scope.
  const error = await t.device.popErrorScope();
  t.expect(t.isInstanceOfError(errorFilter, error));
});

g.test('current_scope').
desc(
  `
Tests that an error does not bubbles to parent scopes when local scope matches.

- Different error types as the current scope
- Different depths of non-capturing filters for the generated error
    `
).
params((u) =>
u.
combine('errorFilter', kGeneratableErrorScopeFilters).
combine('stackDepth', [1, 10, 100, 1000, 100000])
).
fn(async (t) => {
  const { errorFilter, stackDepth } = t.params;

  // Push a bunch of error filters onto the stack
  for (let i = 0; i < stackDepth; i++) {
    t.device.pushErrorScope(kErrorScopeFilters[i % kErrorScopeFilters.length]);
  }

  // Current scope should catch the error immediately.
  t.device.pushErrorScope(errorFilter);
  t.generateError(errorFilter);
  const error = await t.device.popErrorScope();
  t.expect(t.isInstanceOfError(errorFilter, error));

  // Remaining scopes shouldn't catch anything.
  const promises = [];
  for (let i = 0; i < stackDepth; i++) {
    promises.push(t.device.popErrorScope());
  }
  const errors = await Promise.all(promises);
  t.expect(errors.every((e) => e === null));
});

g.test('balanced_siblings').
desc(
  `
Tests that sibling error scopes need to be balanced.

- Different error types as the current scope
- Different number of sibling errors
    `
).
params((u) =>
u.combine('errorFilter', kErrorScopeFilters).combine('numErrors', [1, 10, 100, 1000])
).
fn(async (t) => {
  const { errorFilter, numErrors } = t.params;

  const promises = [];
  for (let i = 0; i < numErrors; i++) {
    t.device.pushErrorScope(errorFilter);
    promises.push(t.device.popErrorScope());
  }

  {
    // Trying to pop an additional non-existing scope should reject.
    const promise = t.device.popErrorScope();
    t.shouldReject('OperationError', promise, { allowMissingStack: true });
  }

  const errors = await Promise.all(promises);
  t.expect(errors.every((e) => e === null));
});

g.test('balanced_nesting').
desc(
  `
Tests that nested error scopes need to be balanced.

- Different error types as the current scope
- Different number of nested errors
    `
).
params((u) =>
u.combine('errorFilter', kErrorScopeFilters).combine('numErrors', [1, 10, 100, 1000])
).
fn(async (t) => {
  const { errorFilter, numErrors } = t.params;

  for (let i = 0; i < numErrors; i++) {
    t.device.pushErrorScope(errorFilter);
  }

  const promises = [];
  for (let i = 0; i < numErrors; i++) {
    promises.push(t.device.popErrorScope());
  }
  const errors = await Promise.all(promises);
  t.expect(errors.every((e) => e === null));

  {
    // Trying to pop an additional non-existing scope should reject.
    const promise = t.device.popErrorScope();
    t.shouldReject('OperationError', promise, { allowMissingStack: true });
  }
});