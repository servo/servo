/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { extractImportantStackTrace } from '../util/stack.js';
export class LogMessageWithStack extends Error {
  constructor(name, ex) {
    super(ex.message);

    _defineProperty(this, "stackHidden", false);

    _defineProperty(this, "timesSeen", 1);

    this.name = name;
    this.stack = ex.stack;
  }
  /** Set a flag so the stack is not printed in toJSON(). */


  setStackHidden() {
    this.stackHidden = true;
  }
  /** Increment the "seen x times" counter. */


  incrementTimesSeen() {
    this.timesSeen++;
  }

  toJSON() {
    let m = this.name + ': ';

    if (!this.stackHidden && this.stack) {
      // this.message is already included in this.stack
      m += extractImportantStackTrace(this);
    } else {
      m += this.message;
    }

    if (this.timesSeen > 1) {
      m += `\n(seen ${this.timesSeen} times with identical stack)`;
    }

    return m;
  }

}
//# sourceMappingURL=log_message.js.map