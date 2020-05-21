/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { extractPublicParams, publicParamsEquals } from './params_utils.js';
import { kPathSeparator } from './query/separators.js';
import { stringifyPublicParams } from './query/stringify_params.js';
import { validQueryPart } from './query/validQueryPart.js';
import { assert } from './util/util.js';
export function makeTestGroup(fixture) {
  return new TestGroup(fixture);
} // Interface for running tests

export function makeTestGroupForUnitTesting(fixture) {
  return new TestGroup(fixture);
}

class TestGroup {
  constructor(fixture) {
    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "seen", new Set());

    _defineProperty(this, "tests", []);

    this.fixture = fixture;
  }

  *iterate() {
    for (const test of this.tests) {
      yield* test.iterate();
    }
  }

  checkName(name) {
    assert( // Shouldn't happen due to the rule above. Just makes sure that treated
    // unencoded strings as encoded strings is OK.
    name === decodeURIComponent(name), `Not decodeURIComponent-idempotent: ${name} !== ${decodeURIComponent(name)}`);
    assert(!this.seen.has(name), `Duplicate test name: ${name}`);
    this.seen.add(name);
  } // TODO: This could take a fixture, too, to override the one for the group.


  test(name) {
    this.checkName(name);
    const parts = name.split(kPathSeparator);

    for (const p of parts) {
      assert(validQueryPart.test(p), `Invalid test name part ${p}; must match ${validQueryPart}`);
    }

    const test = new TestBuilder(parts, this.fixture);
    this.tests.push(test);
    return test;
  }

  checkCaseNamesAndDuplicates() {
    for (const test of this.tests) {
      test.checkCaseNamesAndDuplicates();
    }
  }

}

class TestBuilder {
  constructor(testPath, fixture) {
    _defineProperty(this, "testPath", void 0);

    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "testFn", void 0);

    _defineProperty(this, "cases", undefined);

    this.testPath = testPath;
    this.fixture = fixture;
  }

  fn(fn) {
    this.testFn = fn;
  }

  checkCaseNamesAndDuplicates() {
    if (this.cases === undefined) {
      return;
    } // This is n^2.


    const seen = [];

    for (const testcase of this.cases) {
      // stringifyPublicParams also checks for invalid params values
      const testcaseString = stringifyPublicParams(testcase);
      assert(!seen.some(x => publicParamsEquals(x, testcase)), `Duplicate public test case params: ${testcaseString}`);
      seen.push(testcase);
    }
  }

  params(casesIterable) {
    assert(this.cases === undefined, 'test case is already parameterized');
    this.cases = Array.from(casesIterable);
    return this;
  }

  *iterate() {
    assert(this.testFn !== undefined, 'internal error');

    for (const params of this.cases || [{}]) {
      yield new RunCaseSpecific(this.testPath, params, this.fixture, this.testFn);
    }
  }

}

class RunCaseSpecific {
  constructor(testPath, params, fixture, fn) {
    _defineProperty(this, "id", void 0);

    _defineProperty(this, "params", void 0);

    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "fn", void 0);

    this.id = {
      test: testPath,
      params: extractPublicParams(params)
    };
    this.params = params;
    this.fixture = fixture;
    this.fn = fn;
  }

  async run(rec) {
    rec.start();

    try {
      const inst = new this.fixture(rec, this.params || {});

      try {
        await inst.init();
        await this.fn(inst);
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
    }

    rec.finish();
  }

}
//# sourceMappingURL=test_group.js.map