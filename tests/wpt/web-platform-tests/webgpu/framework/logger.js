/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { makeQueryString } from './url_query.js';
import { extractPublicParams } from './url_query.js';
import { getStackTrace, now } from './util/index.js';
import { version } from './version.js';
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
export class TestCaseRecorder {
  constructor(result) {
    _defineProperty(this, "result", void 0);

    _defineProperty(this, "failed", false);

    _defineProperty(this, "warned", false);

    _defineProperty(this, "startTime", -1);

    _defineProperty(this, "logs", []);

    _defineProperty(this, "debugging", false);

    this.result = result;
  }

  start(debug = false) {
    this.startTime = now();
    this.logs = [];
    this.failed = false;
    this.warned = false;
    this.debugging = debug;
  }

  finish() {
    if (this.startTime < 0) {
      throw new Error('finish() before start()');
    }

    const endTime = now(); // Round to next microsecond to avoid storing useless .xxxx00000000000002 in results.

    this.result.timems = Math.ceil((endTime - this.startTime) * 1000) / 1000;
    this.result.status = this.failed ? 'fail' : this.warned ? 'warn' : 'pass';
    this.result.logs = this.logs;
    this.debugging = false;
  }

  debug(msg) {
    if (!this.debugging) {
      return;
    }

    this.log('DEBUG: ' + msg);
  }

  log(msg) {
    this.logs.push(msg);
  }

  warn(msg) {
    this.warned = true;
    let m = 'WARN';

    if (msg) {
      m += ': ' + msg;
    }

    m += ' ' + getStackTrace(new Error());
    this.log(m);
  }

  fail(msg) {
    this.failed = true;
    let m = 'FAIL';

    if (msg) {
      m += ': ' + msg;
    }

    m += ' ' + getStackTrace(new Error());
    this.log(m);
  }

  threw(e) {
    this.failed = true;
    this.log('EXCEPTION: ' + e.name + ': ' + e.message + '\n' + getStackTrace(e));
  }

}
//# sourceMappingURL=logger.js.map