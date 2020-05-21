/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { version } from '../version.js';
import { TestCaseRecorder } from './test_case_recorder.js';
export class Logger {
  constructor(debug) {
    _defineProperty(this, "debug", void 0);

    _defineProperty(this, "results", new Map());

    this.debug = debug;
  }

  record(name) {
    const result = {
      status: 'running',
      timems: -1
    };
    this.results.set(name, result);
    return [new TestCaseRecorder(result, this.debug), result];
  }

  asJSON(space) {
    return JSON.stringify({
      version,
      results: Array.from(this.results)
    }, undefined, space);
  }

}
//# sourceMappingURL=logger.js.map