/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { extractImportantStackTrace } from '../stack.js';
export class LogMessageWithStack extends Error {
  stackHiddenMessage = undefined;

  constructor(name, ex) {
    super(ex.message);

    this.name = name;
    this.stack = ex.stack;
    if ('extra' in ex) {
      this.extra = ex.extra;
    }
  }

  /** Set a flag so the stack is not printed in toJSON(). */
  setStackHidden(stackHiddenMessage) {
    this.stackHiddenMessage ??= stackHiddenMessage;
  }

  toJSON() {
    let m = this.name;
    if (this.message) m += ': ' + this.message;
    if (this.stack) {
      if (this.stackHiddenMessage === undefined) {
        m += '\n' + extractImportantStackTrace(this);
      } else if (this.stackHiddenMessage) {
        m += `\n  at (elided: ${this.stackHiddenMessage})`;
      }
    }
    return m;
  }
}

/**
 * Returns a string, nicely indented, for debug logs.
 * This is used in the cmdline and wpt runtimes. In WPT, it shows up in the `*-actual.txt` file.
 */
export function prettyPrintLog(log) {
  return '  - ' + log.toJSON().replace(/\n/g, '\n    ');
}
