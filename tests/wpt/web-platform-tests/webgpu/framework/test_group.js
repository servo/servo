/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { allowedTestNameCharacters } from './allowed_characters.js';
import { paramsEquals } from './params/index.js';
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
    if (!validNames.test(name)) {
      throw new Error(`Invalid test name ${name}; must match [${validNames}]+`);
    }

    if (name !== decodeURIComponent(name)) {
      // Shouldn't happen due to the rule above. Just makes sure that treated
      // unencoded strings as encoded strings is OK.
      throw new Error(`Not decodeURIComponent-idempotent: ${name} !== ${decodeURIComponent(name)}`);
    }

    if (this.seen.has(name)) {
      throw new Error('Duplicate test name');
    }

    this.seen.add(name);
  } // TODO: This could take a fixture, too, to override the one for the group.


  test(name, fn) {
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
    if (this.cases !== null) {
      throw new Error('test case is already parameterized');
    }

    const cases = Array.from(specs);
    const seen = []; // This is n^2.

    for (const spec of cases) {
      if (seen.some(x => paramsEquals(x, spec))) {
        throw new Error('Duplicate test case params');
      }

      seen.push(spec);
    }

    this.cases = cases;
  }

  *iterate(rec) {
    for (const params of this.cases || [null]) {
      yield new RunCaseSpecific(rec, {
        test: this.name,
        params
      }, this.fixture, this.fn);
    }
  }

}

class RunCaseSpecific {
  constructor(recorder, id, fixture, fn) {
    _defineProperty(this, "id", void 0);

    _defineProperty(this, "recorder", void 0);

    _defineProperty(this, "fixture", void 0);

    _defineProperty(this, "fn", void 0);

    this.recorder = recorder;
    this.id = id;
    this.fixture = fixture;
    this.fn = fn;
  }

  async run() {
    const [rec, res] = this.recorder.record(this.id.test, this.id.params);
    rec.start();

    try {
      const inst = new this.fixture(rec, this.id.params || {});
      await inst.init();
      await this.fn(inst);
      inst.finalize();
    } catch (e) {
      rec.threw(e);
    }

    rec.finish();
    return res;
  }

}
//# sourceMappingURL=test_group.js.map