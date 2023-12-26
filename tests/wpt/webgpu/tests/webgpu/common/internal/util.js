/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /**
 * Error without a stack, which can be used to fatally exit from `tool/` scripts with a
 * user-friendly message (and no confusing stack).
 */export class StacklessError extends Error {constructor(message) {
    super(message);
    this.stack = undefined;
  }
}