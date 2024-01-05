/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { ValidationTest } from '../api/validation/validation_test.js';export class CompatibilityTest extends ValidationTest {
  async init() {
    await super.init();
    if (!this.isCompatibility) {
      this.skip('compatibility tests do not work on non-compatibility mode');
    }
  }
}