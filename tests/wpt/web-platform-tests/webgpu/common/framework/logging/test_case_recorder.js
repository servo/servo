/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { SkipTestCase } from '../fixture.js';
import { now, assert } from '../util/util.js';
import { LogMessageWithStack } from './log_message.js';
var LogSeverity;

(function (LogSeverity) {
  LogSeverity[LogSeverity["Pass"] = 0] = "Pass";
  LogSeverity[LogSeverity["Skip"] = 1] = "Skip";
  LogSeverity[LogSeverity["Warn"] = 2] = "Warn";
  LogSeverity[LogSeverity["ExpectFailed"] = 3] = "ExpectFailed";
  LogSeverity[LogSeverity["ValidationFailed"] = 4] = "ValidationFailed";
  LogSeverity[LogSeverity["ThrewException"] = 5] = "ThrewException";
})(LogSeverity || (LogSeverity = {}));

const kMaxLogStacks = 2;
/** Holds onto a LiveTestCaseResult owned by the Logger, and writes the results into it. */

export class TestCaseRecorder {
  /** Used to dedup log messages which have identical stacks. */
  constructor(result, debugging) {
    _defineProperty(this, "result", void 0);

    _defineProperty(this, "maxLogSeverity", LogSeverity.Pass);

    _defineProperty(this, "startTime", -1);

    _defineProperty(this, "logs", []);

    _defineProperty(this, "logLinesAtCurrentSeverity", 0);

    _defineProperty(this, "debugging", false);

    _defineProperty(this, "messagesForPreviouslySeenStacks", new Map());

    this.result = result;
    this.debugging = debugging;
  }

  start() {
    assert(this.startTime < 0, 'TestCaseRecorder cannot be reused');
    this.startTime = now();
  }

  finish() {
    assert(this.startTime >= 0, 'finish() before start()');
    const timeMilliseconds = now() - this.startTime; // Round to next microsecond to avoid storing useless .xxxx00000000000002 in results.

    this.result.timems = Math.ceil(timeMilliseconds * 1000) / 1000; // Convert numeric enum back to string (but expose 'exception' as 'fail')

    this.result.status = this.maxLogSeverity === LogSeverity.Pass ? 'pass' : this.maxLogSeverity === LogSeverity.Skip ? 'skip' : this.maxLogSeverity === LogSeverity.Warn ? 'warn' : 'fail'; // Everything else is an error

    this.result.logs = this.logs;
  }

  injectResult(injectedResult) {
    Object.assign(this.result, injectedResult);
  }

  debug(ex) {
    if (!this.debugging) {
      return;
    }

    const logMessage = new LogMessageWithStack('DEBUG', ex);
    logMessage.setStackHidden();
    this.logImpl(LogSeverity.Pass, logMessage);
  }

  skipped(ex) {
    this.logImpl(LogSeverity.Skip, new LogMessageWithStack('SKIP', ex));
  }

  warn(ex) {
    this.logImpl(LogSeverity.Warn, new LogMessageWithStack('WARN', ex));
  }

  expectationFailed(ex) {
    this.logImpl(LogSeverity.ExpectFailed, new LogMessageWithStack('EXPECTATION FAILED', ex));
  }

  validationFailed(ex) {
    this.logImpl(LogSeverity.ValidationFailed, new LogMessageWithStack('VALIDATION FAILED', ex));
  }

  threw(ex) {
    if (ex instanceof SkipTestCase) {
      this.skipped(ex);
      return;
    }

    this.logImpl(LogSeverity.ThrewException, new LogMessageWithStack('EXCEPTION', ex));
  }

  logImpl(level, logMessage) {
    // Deduplicate errors with the exact same stack
    if (logMessage.stack) {
      const seen = this.messagesForPreviouslySeenStacks.get(logMessage.stack);

      if (seen) {
        seen.incrementTimesSeen();
        return;
      }

      this.messagesForPreviouslySeenStacks.set(logMessage.stack, logMessage);
    } // Mark printStack=false for all logs except 2 at the highest severity


    if (level > this.maxLogSeverity) {
      this.logLinesAtCurrentSeverity = 0;
      this.maxLogSeverity = level;

      if (!this.debugging) {
        // Go back and turn off printStack for everything of a lower log level
        for (const log of this.logs) {
          log.setStackHidden();
        }
      }
    }

    if (level < this.maxLogSeverity || this.logLinesAtCurrentSeverity >= kMaxLogStacks) {
      if (!this.debugging) {
        logMessage.setStackHidden();
      }
    }

    this.logs.push(logMessage);
    this.logLinesAtCurrentSeverity++;
  }

}
//# sourceMappingURL=test_case_recorder.js.map