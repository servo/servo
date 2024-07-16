/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import {
  SkipTestCase,

  UnexpectedPassError } from


'../framework/fixture.js';
import {

  builderIterateCasesWithSubcases,
  kUnitCaseParamsBuilder } from


'../framework/params_builder.js';
import { globalTestConfig } from '../framework/test_config.js';

import { TestCaseRecorder } from '../internal/logging/test_case_recorder.js';
import { extractPublicParams, mergeParams } from '../internal/params_utils.js';
import { compareQueries, Ordering } from '../internal/query/compare.js';
import {

  TestQueryMultiTest,
  TestQuerySingleCase } from

'../internal/query/query.js';
import { kPathSeparator } from '../internal/query/separators.js';
import {
  stringifyPublicParams,
  stringifyPublicParamsUniquely } from
'../internal/query/stringify_params.js';
import { validQueryPart } from '../internal/query/validQueryPart.js';

import { assert, unreachable } from '../util/util.js';

import { logToWebSocket } from './websocket_logger.js';






















// Interface for defining tests



export function makeTestGroup(fixture) {
  return new TestGroup(fixture);
}

// Interfaces for running tests













export function makeTestGroupForUnitTesting(
fixture)
{
  return new TestGroup(fixture);
}

/** The maximum allowed length of a test query string. Checked by tools/validate. */
export const kQueryMaxLength = 375;

/** Parameter name for batch number (see also TestBuilder.batch). */
const kBatchParamName = 'batch__';








export class TestGroup {

  seen = new Set();
  tests = [];

  constructor(fixture) {
    this.fixture = fixture;
  }

  iterate() {
    return this.tests;
  }

  checkName(name) {
    assert(
      // Shouldn't happen due to the rule above. Just makes sure that treating
      // unencoded strings as encoded strings is OK.
      name === decodeURIComponent(name),
      `Not decodeURIComponent-idempotent: ${name} !== ${decodeURIComponent(name)}`
    );
    assert(!this.seen.has(name), `Duplicate test name: ${name}`);

    this.seen.add(name);
  }

  test(name) {
    const testCreationStack = new Error(`Test created: ${name}`);

    this.checkName(name);

    const parts = name.split(kPathSeparator);
    for (const p of parts) {
      assert(validQueryPart.test(p), `Invalid test name part ${p}; must match ${validQueryPart}`);
    }

    const test = new TestBuilder(parts, this.fixture, testCreationStack);
    this.tests.push(test);
    return test;
  }

  validate(fileQuery) {
    for (const test of this.tests) {
      const testQuery = new TestQueryMultiTest(
        fileQuery.suite,
        fileQuery.filePathParts,
        test.testPath
      );
      test.validate(testQuery);
    }
  }

  collectNonEmptyTests() {
    const testPaths = [];
    for (const test of this.tests) {
      if (test.computeCaseCount() > 0) {
        testPaths.push({ testPath: test.testPath });
      }
    }
    return testPaths;
  }
}
















































































class TestBuilder {








  testCases = undefined;
  batchSize = 0;

  constructor(testPath, fixture, testCreationStack) {
    this.testPath = testPath;
    this.isUnimplemented = false;
    this.fixture = fixture;
    this.testCreationStack = testCreationStack;
  }

  desc(description) {
    this.description = description.trim();
    return this;
  }

  specURL(_url) {
    return this;
  }

  beforeAllSubcases(fn) {
    assert(this.beforeFn === undefined);
    this.beforeFn = fn;
    return this;
  }

  fn(fn) {

    // MAINTENANCE_TODO: add "TODO" if there's no description? (and make sure it only ends up on
    // actual tests, not on test parents in the tree, which is what happens if you do it here, not
    // sure why)
    assert(this.testFn === undefined);
    this.testFn = fn;
  }

  batch(b) {
    this.batchSize = b;
    return this;
  }

  unimplemented() {
    assert(this.testFn === undefined);

    this.description =
    (this.description ? this.description + '\n\n' : '') + 'TODO: .unimplemented()';
    this.isUnimplemented = true;

    // Use the beforeFn to skip the test, so we don't have to iterate the subcases.
    this.beforeFn = () => {
      throw new SkipTestCase('test unimplemented');
    };
    this.testFn = () => {};
  }

  /** Perform various validation/"lint" chenks. */
  validate(testQuery) {
    const testPathString = this.testPath.join(kPathSeparator);
    assert(this.testFn !== undefined, () => {
      let s = `Test is missing .fn(): ${testPathString}`;
      if (this.testCreationStack.stack) {
        s += `\n-> test created at:\n${this.testCreationStack.stack}`;
      }
      return s;
    });

    assert(
      testQuery.toString().length <= kQueryMaxLength,
      () =>
      `Test query ${testQuery} is too long. Max length is ${kQueryMaxLength} characters. Please shorten names or reduce parameters.`
    );

    if (this.testCases === undefined) {
      return;
    }

    const seen = new Set();
    for (const [caseParams, subcases] of builderIterateCasesWithSubcases(this.testCases, null)) {
      const caseQuery = new TestQuerySingleCase(
        testQuery.suite,
        testQuery.filePathParts,
        testQuery.testPathParts,
        caseParams
      ).toString();
      assert(
        caseQuery.length <= kQueryMaxLength,
        () =>
        `Case query ${caseQuery} is too long. Max length is ${kQueryMaxLength} characters. Please shorten names or reduce parameters.`
      );

      for (const subcaseParams of subcases ?? [{}]) {
        const params = mergeParams(caseParams, subcaseParams);
        assert(this.batchSize === 0 || !(kBatchParamName in params));

        // stringifyPublicParams also checks for invalid params values
        let testcaseString;
        try {
          testcaseString = stringifyPublicParams(params);
        } catch (e) {
          throw new Error(`${e}: ${testPathString}`);
        }

        // A (hopefully) unique representation of a params value.
        const testcaseStringUnique = stringifyPublicParamsUniquely(params);
        assert(
          !seen.has(testcaseStringUnique),
          `Duplicate public test case+subcase params for test ${testPathString}: ${testcaseString} (${caseQuery})`
        );
        seen.add(testcaseStringUnique);
      }
    }
  }

  computeCaseCount() {
    if (this.testCases === undefined) {
      return 1;
    }

    let caseCount = 0;
    for (const [_caseParams, _subcases] of builderIterateCasesWithSubcases(this.testCases, null)) {
      caseCount++;
    }
    return caseCount;
  }

  params(
  cases)
  {
    assert(this.testCases === undefined, 'test case is already parameterized');
    if (cases instanceof Function) {
      this.testCases = cases(kUnitCaseParamsBuilder);
    } else {
      this.testCases = cases;
    }
    return this;
  }

  paramsSimple(cases) {
    assert(this.testCases === undefined, 'test case is already parameterized');
    this.testCases = kUnitCaseParamsBuilder.combineWithParams(cases);
    return this;
  }

  paramsSubcasesOnly(
  subcases)
  {
    if (subcases instanceof Function) {
      return this.params(subcases(kUnitCaseParamsBuilder.beginSubcases()));
    } else {
      return this.params(kUnitCaseParamsBuilder.beginSubcases().combineWithParams(subcases));
    }
  }

  makeCaseSpecific(params, subcases) {
    assert(this.testFn !== undefined, 'No test function (.fn()) for test');
    return new RunCaseSpecific(
      this.testPath,
      params,
      this.isUnimplemented,
      subcases,
      this.fixture,
      this.testFn,
      this.beforeFn,
      this.testCreationStack
    );
  }

  *iterate(caseFilter) {
    this.testCases ??= kUnitCaseParamsBuilder;

    // Remove the batch__ from the caseFilter because the params builder doesn't
    // know about it (we don't add it until later in this function).
    let filterToBatch;
    const caseFilterWithoutBatch = caseFilter ? { ...caseFilter } : null;
    if (caseFilterWithoutBatch && kBatchParamName in caseFilterWithoutBatch) {
      const batchParam = caseFilterWithoutBatch[kBatchParamName];
      assert(typeof batchParam === 'number');
      filterToBatch = batchParam;
      delete caseFilterWithoutBatch[kBatchParamName];
    }

    for (const [caseParams, subcases] of builderIterateCasesWithSubcases(
      this.testCases,
      caseFilterWithoutBatch
    )) {
      // If batches are not used, yield just one case.
      if (this.batchSize === 0 || subcases === undefined) {
        yield this.makeCaseSpecific(caseParams, subcases);
        continue;
      }

      // Same if there ends up being only one batch.
      const subcaseArray = Array.from(subcases);
      if (subcaseArray.length <= this.batchSize) {
        yield this.makeCaseSpecific(caseParams, subcaseArray);
        continue;
      }

      // There are multiple batches. Helper function for this case:
      const makeCaseForBatch = (batch) => {
        const sliceStart = batch * this.batchSize;
        return this.makeCaseSpecific(
          { ...caseParams, [kBatchParamName]: batch },
          subcaseArray.slice(sliceStart, Math.min(subcaseArray.length, sliceStart + this.batchSize))
        );
      };

      // If we filter to just one batch, yield it.
      if (filterToBatch !== undefined) {
        yield makeCaseForBatch(filterToBatch);
        continue;
      }

      // Finally, if not, yield all of the batches.
      for (let batch = 0; batch * this.batchSize < subcaseArray.length; ++batch) {
        yield makeCaseForBatch(batch);
      }
    }
  }
}

class RunCaseSpecific {










  constructor(
  testPath,
  params,
  isUnimplemented,
  subcases,
  fixture,
  fn,
  beforeFn,
  testCreationStack)
  {
    this.id = { test: testPath, params: extractPublicParams(params) };
    this.isUnimplemented = isUnimplemented;
    this.params = params;
    this.subcases = subcases;
    this.fixture = fixture;
    this.fn = fn;
    this.beforeFn = beforeFn;
    this.testCreationStack = testCreationStack;
  }

  computeSubcaseCount() {
    if (this.subcases) {
      let count = 0;
      for (const _subcase of this.subcases) {
        count++;
      }
      return count;
    } else {
      return 1;
    }
  }

  async runTest(
  rec,
  sharedState,
  params,
  throwSkip,
  expectedStatus)
  {
    try {
      rec.beginSubCase();
      if (expectedStatus === 'skip') {
        throw new SkipTestCase('Skipped by expectations');
      }

      const inst = new this.fixture(sharedState, rec, params);
      try {
        await inst.init();
        await this.fn(inst);
        rec.passed();
      } finally {
        // Runs as long as constructor succeeded, even if initialization or the test failed.
        await inst.finalize();
      }
    } catch (ex) {
      // There was an exception from constructor, init, test, or finalize.
      // An error from init or test may have been a SkipTestCase.
      // An error from finalize may have been an eventualAsyncExpectation failure
      // or unexpected validation/OOM error from the GPUDevice.
      rec.threw(ex);
      if (throwSkip && ex instanceof SkipTestCase) {
        throw ex;
      }
    } finally {
      try {
        rec.endSubCase(expectedStatus);
      } catch (ex) {
        assert(ex instanceof UnexpectedPassError);
        ex.message = `Testcase passed unexpectedly.`;
        ex.stack = this.testCreationStack.stack;
        rec.warn(ex);
      }
    }
  }

  async run(
  rec,
  selfQuery,
  expectations)
  {
    const getExpectedStatus = (selfQueryWithSubParams) => {
      let didSeeFail = false;
      for (const exp of expectations) {
        const ordering = compareQueries(exp.query, selfQueryWithSubParams);
        if (ordering === Ordering.Unordered || ordering === Ordering.StrictSubset) {
          continue;
        }

        switch (exp.expectation) {
          // Skip takes precedence. If there is any expectation indicating a skip,
          // signal it immediately.
          case 'skip':
            return 'skip';
          case 'fail':
            // Otherwise, indicate that we might expect a failure.
            didSeeFail = true;
            break;
          default:
            unreachable();
        }
      }
      return didSeeFail ? 'fail' : 'pass';
    };

    const { testHeartbeatCallback, maxSubcasesInFlight } = globalTestConfig;
    try {
      rec.start();
      const sharedState = this.fixture.MakeSharedState(rec, this.params);
      try {
        await sharedState.init();
        if (this.beforeFn) {
          await this.beforeFn(sharedState);
        }
        await sharedState.postInit();
        testHeartbeatCallback();

        let allPreviousSubcasesFinalizedPromise = Promise.resolve();
        if (this.subcases) {
          let totalCount = 0;
          let skipCount = 0;

          // If there are too many subcases in flight, starting the next subcase will register
          // `resolvePromiseBlockingSubcase` and wait until `subcaseFinishedCallback` is called.
          let subcasesInFlight = 0;
          let resolvePromiseBlockingSubcase = undefined;
          const subcaseFinishedCallback = () => {
            subcasesInFlight -= 1;
            // If there is any subcase waiting on a previous subcase to finish,
            // unblock it now, and clear the resolve callback.
            if (resolvePromiseBlockingSubcase) {
              resolvePromiseBlockingSubcase();
              resolvePromiseBlockingSubcase = undefined;
            }
          };

          for (const subParams of this.subcases) {
            // Make a recorder that will defer all calls until `allPreviousSubcasesFinalizedPromise`
            // resolves. Waiting on `allPreviousSubcasesFinalizedPromise` ensures that
            // logs from all the previous subcases have been flushed before flushing new logs.
            const subcasePrefix = 'subcase: ' + stringifyPublicParams(subParams);
            const subRec = new Proxy(rec, {
              get: (target, k) => {
                const prop = TestCaseRecorder.prototype[k];
                if (typeof prop === 'function') {
                  testHeartbeatCallback();
                  return function (...args) {
                    void allPreviousSubcasesFinalizedPromise.then(() => {
                      // Prepend the subcase name to all error messages.
                      for (const arg of args) {
                        if (arg instanceof Error) {
                          try {
                            arg.message = subcasePrefix + '\n' + arg.message;
                          } catch {
                            // If that fails (e.g. on DOMException), try to put it in the stack:
                            let stack = subcasePrefix;
                            if (arg.stack) stack += '\n' + arg.stack;
                            try {
                              arg.stack = stack;
                            } catch {

                              // If that fails too, just silence it.
                            }}
                        }
                      }


                      const rv = prop.apply(target, args);
                      // Because this proxy executes functions in a deferred manner,
                      // it should never be used for functions that need to return a value.
                      assert(rv === undefined);
                    });
                  };
                }
                return prop;
              }
            });

            const params = mergeParams(this.params, subParams);
            const subcaseQuery = new TestQuerySingleCase(
              selfQuery.suite,
              selfQuery.filePathParts,
              selfQuery.testPathParts,
              params
            );

            // Limit the maximum number of subcases in flight.
            if (subcasesInFlight >= maxSubcasesInFlight) {
              await new Promise((resolve) => {
                // There should only be one subcase waiting at a time.
                assert(resolvePromiseBlockingSubcase === undefined);
                resolvePromiseBlockingSubcase = resolve;
              });
            }

            subcasesInFlight += 1;
            // Runs async without waiting so that subsequent subcases can start.
            // All finalization steps will be waited on at the end of the testcase.
            const finalizePromise = this.runTest(
              subRec,
              sharedState,
              params,
              /* throwSkip */true,
              getExpectedStatus(subcaseQuery)
            ).
            then(() => {
              subRec.info(new Error('OK'));
            }).
            catch((ex) => {
              if (ex instanceof SkipTestCase) {
                // Convert SkipTestCase to info messages
                ex.message = 'subcase skipped: ' + ex.message;
                subRec.info(ex);
                ++skipCount;
              } else {
                // Since we are catching all error inside runTest(), this should never happen
                subRec.threw(ex);
              }
            }).
            finally(subcaseFinishedCallback);

            allPreviousSubcasesFinalizedPromise = allPreviousSubcasesFinalizedPromise.then(
              () => finalizePromise
            );
            ++totalCount;
          }

          // Wait for all subcases to finalize and report their results.
          await allPreviousSubcasesFinalizedPromise;

          if (skipCount === totalCount) {
            rec.skipped(new SkipTestCase('all subcases were skipped'));
          }
        } else {
          await this.runTest(
            rec,
            sharedState,
            this.params,
            /* throwSkip */false,
            getExpectedStatus(selfQuery)
          );
        }
      } finally {
        testHeartbeatCallback();
        // Runs as long as the shared state constructor succeeded, even if initialization or a test failed.
        await sharedState.finalize();
        testHeartbeatCallback();
      }
    } catch (ex) {
      // There was an exception from sharedState/fixture constructor, init, beforeFn, or test.
      // An error from beforeFn may have been SkipTestCase.
      // An error from finalize may have been an eventualAsyncExpectation failure
      // or unexpected validation/OOM error from the GPUDevice.
      rec.threw(ex);
    } finally {
      rec.finish();

      const msg = {
        q: selfQuery.toString(),
        timems: rec.result.timems,
        nonskippedSubcaseCount: rec.nonskippedSubcaseCount
      };
      logToWebSocket(JSON.stringify(msg));
    }
  }
}