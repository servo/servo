/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { version } from '../version.js';
import { TestCaseRecorder } from './test_case_recorder.js';

export class Logger {
  static globalDebugMode = false;

  results = new Map();

  constructor({ overrideDebugMode } = {}) {
    this.overriddenDebugMode = overrideDebugMode;
  }

  record(name) {
    const result = { status: 'running', timems: -1 };
    this.results.set(name, result);
    return [
      new TestCaseRecorder(result, this.overriddenDebugMode ?? Logger.globalDebugMode),
      result,
    ];
  }

  asJSON(space) {
    return JSON.stringify({ version, results: Array.from(this.results) }, undefined, space);
  }
}
