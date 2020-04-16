/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { assert } from './util/util.js';
export class SkipTestCase extends Error {} // A Fixture is a class used to instantiate each test case at run time.
// A new instance of the Fixture is created for every single test case
// (i.e. every time the test function is run).

export class Fixture {
  constructor(rec, params) {
    _defineProperty(this, "params", void 0);

    _defineProperty(this, "rec", void 0);

    _defineProperty(this, "eventualExpectations", []);

    _defineProperty(this, "numOutstandingAsyncExpectations", 0);

    this.rec = rec;
    this.params = params;
  } // This has to be a member function instead of an async `createFixture` function, because
  // we need to be able to ergonomically override it in subclasses.


  async init() {}

  debug(msg) {
    this.rec.debug(new Error(msg));
  }

  skip(msg) {
    throw new SkipTestCase(msg);
  }

  async finalize() {
    assert(this.numOutstandingAsyncExpectations === 0, 'there were outstanding asynchronous expectations (e.g. shouldReject) at the end of the test');
    await Promise.all(this.eventualExpectations);
  }

  warn(msg) {
    this.rec.warn(new Error(msg));
  }

  fail(msg) {
    this.rec.fail(new Error(msg));
  }

  async immediateAsyncExpectation(fn) {
    this.numOutstandingAsyncExpectations++;
    const ret = await fn();
    this.numOutstandingAsyncExpectations--;
    return ret;
  }

  eventualAsyncExpectation(fn) {
    const promise = fn(new Error());
    this.eventualExpectations.push(promise);
    return promise;
  }

  expectErrorValue(expectedName, ex, niceStack) {
    if (!(ex instanceof Error)) {
      niceStack.message = 'THREW non-error value, of type ' + typeof ex + niceStack.message;
      this.rec.fail(niceStack);
      return;
    }

    const actualName = ex.name;

    if (actualName !== expectedName) {
      niceStack.message = `THREW ${actualName}, instead of ${expectedName}` + niceStack.message;
      this.rec.fail(niceStack);
    } else {
      niceStack.message = 'OK: threw ' + actualName + niceStack.message;
      this.rec.debug(niceStack);
    }
  }

  shouldReject(expectedName, p, msg) {
    this.eventualAsyncExpectation(async niceStack => {
      const m = msg ? ': ' + msg : '';

      try {
        await p;
        niceStack.message = 'DID NOT THROW' + m;
        this.rec.fail(niceStack);
      } catch (ex) {
        niceStack.message = m;
        this.expectErrorValue(expectedName, ex, niceStack);
      }
    });
  }

  shouldThrow(expectedName, fn, msg) {
    const m = msg ? ': ' + msg : '';

    try {
      fn();
      this.rec.fail(new Error('DID NOT THROW' + m));
    } catch (ex) {
      this.expectErrorValue(expectedName, ex, new Error(m));
    }
  }

  expect(cond, msg) {
    if (cond) {
      const m = msg ? ': ' + msg : '';
      this.rec.debug(new Error('expect OK' + m));
    } else {
      this.rec.fail(new Error(msg));
    }

    return cond;
  }

}
//# sourceMappingURL=fixture.js.map