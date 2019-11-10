/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { GPUTest } from '../gpu_test.js';
export class ValidationTest extends GPUTest {
  async getErrorBuffer() {
    this.device.pushErrorScope('validation');
    const errorBuffer = this.device.createBuffer({
      size: 1024,
      usage: 0xffff // Invalid GPUBufferUsage

    });
    await this.device.popErrorScope();
    return errorBuffer;
  }

  expectValidationError(fn, shouldError = true) {
    // If no error is expected, we let the scope surrounding the test catch it.
    if (shouldError === false) {
      fn();
      return;
    }

    this.device.pushErrorScope('validation');
    fn();
    const promise = this.device.popErrorScope();
    this.eventualAsyncExpectation(async niceStack => {
      const gpuValidationError = await promise;

      if (!gpuValidationError) {
        niceStack.message = 'Validation error was expected.';
        this.rec.fail(niceStack);
      } else if (gpuValidationError instanceof GPUValidationError) {
        niceStack.message = `Captured validation error - ${gpuValidationError.message}`;
        this.rec.debug(niceStack);
      }
    });
  }

}
//# sourceMappingURL=validation_test.js.map