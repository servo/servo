/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { globalTestConfig } from '../../framework/test_config.js';import { version } from '../version.js';

import { TestCaseRecorder } from './test_case_recorder.js';



export class Logger {

  results = new Map();

  constructor({ overrideDebugMode } = {}) {
    this.overriddenDebugMode = overrideDebugMode;
  }

  record(name) {
    const result = { status: 'running', timems: -1 };
    this.results.set(name, result);
    return [
    new TestCaseRecorder(result, this.overriddenDebugMode ?? globalTestConfig.enableDebugLogs),
    result];

  }

  asJSON(space) {
    return JSON.stringify({ version, results: Array.from(this.results) }, undefined, space);
  }
}