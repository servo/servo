/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { ValidationTest } from '../api/validation/validation_test.js';export class CompatibilityTest extends ValidationTest {
  async init() {
    await super.init();
  }

  /**
   * Expect a validation error inside the callback.
   * except when not in compat mode.
   *
   * Tests should always do just one WebGPU call in the callback, to make sure that's what's tested.
   */
  expectValidationErrorInCompatibilityMode(fn, shouldError = true) {
    this.expectValidationError(fn, this.isCompatibility && shouldError);
  }

  /**
   * Expect the specified WebGPU error to be generated when running the provided function
   * except when not in compat mode.
   */
  expectGPUErrorInCompatibilityMode(
  filter,
  fn,
  shouldError = true)
  {
    return this.expectGPUError(filter, fn, this.isCompatibility && shouldError);
  }
}