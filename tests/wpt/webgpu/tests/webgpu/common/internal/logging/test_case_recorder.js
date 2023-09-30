/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { SkipTestCase, UnexpectedPassError } from '../../framework/fixture.js';
import { globalTestConfig } from '../../framework/test_config.js';
import { now, assert } from '../../util/util.js';

import { LogMessageWithStack } from './log_message.js';
var LogSeverity;
(function (LogSeverity) {
  LogSeverity[(LogSeverity['Pass'] = 0)] = 'Pass';
  LogSeverity[(LogSeverity['Skip'] = 1)] = 'Skip';
  LogSeverity[(LogSeverity['Warn'] = 2)] = 'Warn';
  LogSeverity[(LogSeverity['ExpectFailed'] = 3)] = 'ExpectFailed';
  LogSeverity[(LogSeverity['ValidationFailed'] = 4)] = 'ValidationFailed';
  LogSeverity[(LogSeverity['ThrewException'] = 5)] = 'ThrewException';
})(LogSeverity || (LogSeverity = {}));

const kMaxLogStacks = 2;
const kMinSeverityForStack = LogSeverity.Warn;

/** Holds onto a LiveTestCaseResult owned by the Logger, and writes the results into it. */
export class TestCaseRecorder {
  nonskippedSubcaseCount = 0;
  inSubCase = false;
  subCaseStatus = LogSeverity.Pass;
  finalCaseStatus = LogSeverity.Pass;
  hideStacksBelowSeverity = kMinSeverityForStack;
  startTime = -1;
  logs = [];
  logLinesAtCurrentSeverity = 0;
  debugging = false;
  /** Used to dedup log messages which have identical stacks. */
  messagesForPreviouslySeenStacks = new Map();

  constructor(result, debugging) {
    this.result = result;
    this.debugging = debugging;
  }

  start() {
    assert(this.startTime < 0, 'TestCaseRecorder cannot be reused');
    this.startTime = now();
  }

  finish() {
    // This is a framework error. If this assert is hit, it won't be localized
    // to a test. The whole test run will fail out.
    assert(this.startTime >= 0, 'internal error: finish() before start()');

    const timeMilliseconds = now() - this.startTime;
    // Round to next microsecond to avoid storing useless .xxxx00000000000002 in results.
    this.result.timems = Math.ceil(timeMilliseconds * 1000) / 1000;

    if (this.finalCaseStatus === LogSeverity.Skip && this.nonskippedSubcaseCount !== 0) {
      this.threw(new Error('internal error: case is "skip" but has nonskipped subcases'));
    }

    // Convert numeric enum back to string (but expose 'exception' as 'fail')
    this.result.status =
      this.finalCaseStatus === LogSeverity.Pass
        ? 'pass'
        : this.finalCaseStatus === LogSeverity.Skip
        ? 'skip'
        : this.finalCaseStatus === LogSeverity.Warn
        ? 'warn'
        : 'fail'; // Everything else is an error

    this.result.logs = this.logs;
  }

  beginSubCase() {
    this.subCaseStatus = LogSeverity.Pass;
    this.inSubCase = true;
  }

  endSubCase(expectedStatus) {
    if (this.subCaseStatus !== LogSeverity.Skip) {
      this.nonskippedSubcaseCount++;
    }
    try {
      if (expectedStatus === 'fail') {
        if (this.subCaseStatus <= LogSeverity.Warn) {
          throw new UnexpectedPassError();
        } else {
          this.subCaseStatus = LogSeverity.Pass;
        }
      }
    } finally {
      this.inSubCase = false;
      if (this.subCaseStatus > this.finalCaseStatus) {
        this.finalCaseStatus = this.subCaseStatus;
      }
    }
  }

  injectResult(injectedResult) {
    Object.assign(this.result, injectedResult);
  }

  debug(ex) {
    if (!this.debugging) return;
    this.logImpl(LogSeverity.Pass, 'DEBUG', ex);
  }

  info(ex) {
    this.logImpl(LogSeverity.Pass, 'INFO', ex);
  }

  skipped(ex) {
    this.logImpl(LogSeverity.Skip, 'SKIP', ex);
  }

  warn(ex) {
    this.logImpl(LogSeverity.Warn, 'WARN', ex);
  }

  expectationFailed(ex) {
    this.logImpl(LogSeverity.ExpectFailed, 'EXPECTATION FAILED', ex);
  }

  validationFailed(ex) {
    this.logImpl(LogSeverity.ValidationFailed, 'VALIDATION FAILED', ex);
  }

  threw(ex) {
    if (ex instanceof SkipTestCase) {
      this.skipped(ex);
      return;
    }
    this.logImpl(LogSeverity.ThrewException, 'EXCEPTION', ex);
  }

  logImpl(level, name, baseException) {
    assert(baseException instanceof Error, 'test threw a non-Error object');
    globalTestConfig.testHeartbeatCallback();
    const logMessage = new LogMessageWithStack(name, baseException);

    // Final case status should be the "worst" of all log entries.
    if (this.inSubCase) {
      if (level > this.subCaseStatus) this.subCaseStatus = level;
    } else {
      if (level > this.finalCaseStatus) this.finalCaseStatus = level;
    }

    // setFirstLineOnly for all logs except `kMaxLogStacks` stacks at the highest severity
    if (level > this.hideStacksBelowSeverity) {
      this.logLinesAtCurrentSeverity = 0;
      this.hideStacksBelowSeverity = level;

      // Go back and setFirstLineOnly for everything of a lower log level
      for (const log of this.logs) {
        log.setStackHidden('below max severity');
      }
    }
    if (level === this.hideStacksBelowSeverity) {
      this.logLinesAtCurrentSeverity++;
    } else if (level < kMinSeverityForStack) {
      logMessage.setStackHidden('');
    } else if (level < this.hideStacksBelowSeverity) {
      logMessage.setStackHidden('below max severity');
    }
    if (this.logLinesAtCurrentSeverity > kMaxLogStacks) {
      logMessage.setStackHidden(`only ${kMaxLogStacks} shown`);
    }

    this.logs.push(logMessage);
  }
}
