/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { allowedTestNameCharacters } from './allowed_characters.js';
import { extractPublicParams, paramsEquals } from './params_utils.js';
import { checkPublicParamType } from './url_query.js';
import { assert } from './util/util.js';
const validNames = new RegExp('^[' + allowedTestNameCharacters + ']+$');
export class TestGroup {
  constructor(fixture) {
    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "seen", new Set());

    _defineProperty(this, "tests", []);

    this.fixture = fixture;
  }

  *iterate(log) {
    for (const test of this.tests) {
      yield* test.iterate(log);
    }
  }

  checkName(name) {
    assert(validNames.test(name), `Invalid test name ${name}; must match [${validNames}]+`);
    assert( // Shouldn't happen due to the rule above. Just makes sure that treated
    // unencoded strings as encoded strings is OK.
    name === decodeURIComponent(name), `Not decodeURIComponent-idempotent: ${name} !== ${decodeURIComponent(name)}`);
    assert(!this.seen.has(name), `Duplicate test name: ${name}`);
    this.seen.add(name);
  } // TODO: This could take a fixture, too, to override the one for the group.


  test(name, fn) {
    // Replace spaces with underscores for readability.
    assert(name.indexOf('_') === -1, 'Invalid test name ${name}: contains underscore (use space)');
    name = name.replace(/ /g, '_');
    this.checkName(name);
    const test = new Test(name, this.fixture, fn);
    this.tests.push(test);
    return test;
  }

} // This test is created when it's inserted, but may be parameterized afterward (.params()).

class Test {
  constructor(name, fixture, fn) {
    _defineProperty(this, "name", void 0);

    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "fn", void 0);

    _defineProperty(this, "cases", null);

    this.name = name;
    this.fixture = fixture;
    this.fn = fn;
  }

  params(specs) {
    assert(this.cases === null, 'test case is already parameterized');
    const cases = Array.from(specs);
    const seen = []; // This is n^2.

    for (const spec of cases) {
      const publicParams = extractPublicParams(spec); // Check type of public params: can only be (currently):
      // number, string, boolean, undefined, number[]

      for (const v of Object.values(publicParams)) {
        checkPublicParamType(v);
      }

      assert(!seen.some(x => paramsEquals(x, publicParams)), 'Duplicate test case params');
      seen.push(publicParams);
    }

    this.cases = cases;
  }

  *iterate(rec) {
    for (const params of this.cases || [null]) {
      yield new RunCaseSpecific(rec, this.name, params, this.fixture, this.fn);
    }
  }

}

class RunCaseSpecific {
  constructor(recorder, test, params, fixture, fn) {
    _defineProperty(this, "id", void 0);

    _defineProperty(this, "params", void 0);

    _defineProperty(this, "recorder", void 0);

    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "fn", void 0);

    this.id = {
      test,
      params: params ? extractPublicParams(params) : null
    };
    this.params = params;
    this.recorder = recorder;
    this.fixture = fixture;
    this.fn = fn;
  }

  async run(debug) {
    const [rec, res] = this.recorder.record(this.id.test, this.id.params);
    rec.start(debug);

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
      // An error from finalize may have been an eventualAsyncExpectation failure.
      rec.threw(ex);
    }

    rec.finish();
    return res;
  }

  injectResult(result) {
    const [, res] = this.recorder.record(this.id.test, this.id.params);
    Object.assign(res, result);
  }

}
//# sourceMappingURL=test_group.js.map