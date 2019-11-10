/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { SkipTestCase } from './fixture.js';
import { makeQueryString } from './url_query.js';
import { extractPublicParams } from './url_query.js';
import { getStackTrace, now } from './util/index.js';
import { version } from './version.js';

class LogMessageWithStack extends Error {
  constructor(name, ex) {
    super(ex.message);
    this.name = name;
    this.stack = ex.stack;
  }

  toJSON() {
    let m = this.name;

    if (this.message) {
      m += ': ' + this.message;
    }

    m += '\n' + getStackTrace(this);
    return m;
  }

}

class LogMessageWithoutStack extends LogMessageWithStack {
  toJSON() {
    return this.message;
  }

}

export class Logger {
  constructor() {
    _defineProperty(this, "results", []);
  }

  record(spec) {
    const result = {
      spec: makeQueryString(spec),
      cases: []
    };
    this.results.push(result);
    return [new TestSpecRecorder(result), result];
  }

  asJSON(space) {
    return JSON.stringify({
      version,
      results: this.results
    }, undefined, space);
  }

}
export class TestSpecRecorder {
  constructor(result) {
    _defineProperty(this, "result", void 0);

    this.result = result;
  }

  record(test, params) {
    const result = {
      test,
      params: params ? extractPublicParams(params) : null,
      status: 'running',
      timems: -1
    };
    this.result.cases.push(result);
    return [new TestCaseRecorder(result), result];
  }

}
var PassState;

(function (PassState) {
  PassState[PassState["pass"] = 0] = "pass";
  PassState[PassState["skip"] = 1] = "skip";
  PassState[PassState["warn"] = 2] = "warn";
  PassState[PassState["fail"] = 3] = "fail";
})(PassState || (PassState = {}));

export class TestCaseRecorder {
  constructor(result) {
    _defineProperty(this, "result", void 0);

    _defineProperty(this, "state", PassState.pass);

    _defineProperty(this, "startTime", -1);

    _defineProperty(this, "logs", []);

    _defineProperty(this, "debugging", false);

    this.result = result;
  }

  start(debug = false) {
    this.startTime = now();
    this.logs = [];
    this.state = PassState.pass;
    this.debugging = debug;
  }

  finish() {
    if (this.startTime < 0) {
      throw new Error('finish() before start()');
    }

    const endTime = now(); // Round to next microsecond to avoid storing useless .xxxx00000000000002 in results.

    this.result.timems = Math.ceil((endTime - this.startTime) * 1000) / 1000;
    this.result.status = PassState[this.state];
    this.result.logs = this.logs;
    this.debugging = false;
  }

  debug(ex) {
    if (!this.debugging) {
      return;
    }

    this.logs.push(new LogMessageWithoutStack('DEBUG', ex));
  }

  warn(ex) {
    this.setState(PassState.warn);
    this.logs.push(new LogMessageWithStack('WARN', ex));
  }

  fail(ex) {
    this.setState(PassState.fail);
    this.logs.push(new LogMessageWithStack('FAIL', ex));
  }

  skipped(ex) {
    this.setState(PassState.skip);
    this.logs.push(new LogMessageWithStack('SKIP', ex));
  }

  threw(ex) {
    if (ex instanceof SkipTestCase) {
      this.skipped(ex);
      return;
    }

    this.setState(PassState.fail);
    this.logs.push(new LogMessageWithStack('EXCEPTION', ex));
  }

  setState(state) {
    this.state = Math.max(this.state, state);
  }

}
//# sourceMappingURL=logger.js.map