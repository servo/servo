/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { SkipTestCase, UnexpectedPassError } from '../../framework/fixture.js';import { globalTestConfig } from '../../framework/test_config.js';import { now, assert } from '../../util/util.js';

import { LogMessageWithStack } from './log_message.js';var


LogSeverity = /*#__PURE__*/function (LogSeverity) {LogSeverity[LogSeverity["NotRun"] = 0] = "NotRun";LogSeverity[LogSeverity["Skip"] = 1] = "Skip";LogSeverity[LogSeverity["Pass"] = 2] = "Pass";LogSeverity[LogSeverity["Warn"] = 3] = "Warn";LogSeverity[LogSeverity["ExpectFailed"] = 4] = "ExpectFailed";LogSeverity[LogSeverity["ValidationFailed"] = 5] = "ValidationFailed";LogSeverity[LogSeverity["ThrewException"] = 6] = "ThrewException";return LogSeverity;}(LogSeverity || {});









const kMaxLogStacks = 2;
const kMinSeverityForStack = LogSeverity.Warn;

function logSeverityToString(status) {
  switch (status) {
    case LogSeverity.NotRun:
      return 'notrun';
    case LogSeverity.Pass:
      return 'pass';
    case LogSeverity.Skip:
      return 'skip';
    case LogSeverity.Warn:
      return 'warn';
    default:
      return 'fail'; // Everything else is an error
  }
}

/** Holds onto a LiveTestCaseResult owned by the Logger, and writes the results into it. */
export class TestCaseRecorder {

  nonskippedSubcaseCount = 0;
  inSubCase = false;
  subCaseStatus = LogSeverity.NotRun;
  finalCaseStatus = LogSeverity.NotRun;
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
    this.result.status = logSeverityToString(this.finalCaseStatus);

    this.result.logs = this.logs;
  }

  beginSubCase() {
    this.subCaseStatus = LogSeverity.NotRun;
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
      this.finalCaseStatus = Math.max(this.finalCaseStatus, this.subCaseStatus);
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
    // We need this to use the lowest LogSeverity so it doesn't override the current severity for this test case.
    this.logImpl(LogSeverity.NotRun, 'INFO', ex);
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

  passed() {
    if (this.inSubCase) {
      this.subCaseStatus = Math.max(this.subCaseStatus, LogSeverity.Pass);
    } else {
      this.finalCaseStatus = Math.max(this.finalCaseStatus, LogSeverity.Pass);
    }
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
      this.subCaseStatus = Math.max(this.subCaseStatus, level);
    } else {
      this.finalCaseStatus = Math.max(this.finalCaseStatus, level);
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