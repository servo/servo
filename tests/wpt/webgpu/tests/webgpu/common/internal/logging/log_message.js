/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { extractImportantStackTrace } from '../stack.js';


export class LogMessageWithStack extends Error {


  stackHiddenMessage = undefined;

  /**
   * Wrap an Error (which was created to capture the stack at that point) into a
   * LogMessageWithStack (which has extra stuff for good log messages).
   *
   * The original `ex.name` is ignored. Inclued it in the `name` parameter if it
   * needs to be preserved.
   */
  static wrapError(name, ex) {
    return new LogMessageWithStack({
      name,
      message: ex.message,
      stackHiddenMessage: undefined,
      stack: ex.stack,
      extra: 'extra' in ex ? ex.extra : undefined
    });
  }

  constructor(o) {
    super(o.message);
    this.name = o.name;
    this.stackHiddenMessage = o.stackHiddenMessage;
    this.stack = o.stack;
    this.extra = o.extra;
  }

  /** Set a flag so the stack is not printed in toJSON(). */
  setStackHidden(stackHiddenMessage) {
    this.stackHiddenMessage ??= stackHiddenMessage;
  }

  /**
   * Print the message for display.
   *
   * Note: This is toJSON instead of toString to make it easy to save logs using JSON.stringify.
   */
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

  /**
   * Flatten the message for sending over a message channel.
   *
   * Note `extra` may get mangled by postMessage.
   */
  toRawData() {
    return {
      name: this.name,
      message: this.message,
      stackHiddenMessage: this.stackHiddenMessage,
      stack: this.stack,
      extra: this.extra
    };
  }
}

/**
 * Returns a string, nicely indented, for debug logs.
 * This is used in the cmdline and wpt runtimes. In WPT, it shows up in the `*-actual.txt` file.
 */
export function prettyPrintLog(log) {
  return '  - ' + log.toJSON().replace(/\n/g, '\n    ');
}