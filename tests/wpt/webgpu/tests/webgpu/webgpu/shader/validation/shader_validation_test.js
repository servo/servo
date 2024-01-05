/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../../common/util/data_tables.js';import { ErrorWithExtra } from '../../../common/util/util.js';import { GPUTest } from '../../gpu_test.js';

/**
 * Base fixture for WGSL shader validation tests.
 */
export class ShaderValidationTest extends GPUTest {
  /**
   * Add a test expectation for whether a createShaderModule call succeeds or not.
   *
   * @example
   * ```ts
   * t.expectCompileResult(true, `wgsl code`); // Expect success
   * t.expectCompileResult(false, `wgsl code`); // Expect validation error with any error string
   * ```
   */
  expectCompileResult(expectedResult, code) {
    let shaderModule;
    this.expectGPUError(
      'validation',
      () => {
        shaderModule = this.device.createShaderModule({ code });
      },
      expectedResult !== true
    );

    const error = new ErrorWithExtra('', () => ({ shaderModule }));
    this.eventualAsyncExpectation(async () => {
      const compilationInfo = await shaderModule.getCompilationInfo();

      // MAINTENANCE_TODO: Pretty-print error messages with source context.
      const messagesLog = compilationInfo.messages.
      map((m) => `${m.lineNum}:${m.linePos}: ${m.type}: ${m.message}`).
      join('\n');
      error.extra.compilationInfo = compilationInfo;

      if (compilationInfo.messages.some((m) => m.type === 'error')) {
        if (expectedResult) {
          error.message = `Unexpected compilationInfo 'error' message.\n` + messagesLog;
          this.rec.validationFailed(error);
        } else {
          error.message = `Found expected compilationInfo 'error' message.\n` + messagesLog;
          this.rec.debug(error);
        }
      } else {
        if (!expectedResult) {
          error.message = `Missing expected compilationInfo 'error' message.\n` + messagesLog;
          this.rec.validationFailed(error);
        } else {
          error.message = `No compilationInfo 'error' messages, as expected.\n` + messagesLog;
          this.rec.debug(error);
        }
      }
    });
  }

  /**
   * Add a test expectation for whether a createShaderModule call issues a warning.
   *
   * @example
   * ```ts
   * t.expectCompileWarning(true, `wgsl code`); // Expect compile success and any warning message
   * t.expectCompileWarning(false, `wgsl code`); // Expect compile success and no warning messages
   * ```
   */
  expectCompileWarning(expectWarning, code) {
    let shaderModule;
    this.expectGPUError(
      'validation',
      () => {
        shaderModule = this.device.createShaderModule({ code });
      },
      false
    );

    const error = new ErrorWithExtra('', () => ({ shaderModule }));
    this.eventualAsyncExpectation(async () => {
      const compilationInfo = await shaderModule.getCompilationInfo();

      // MAINTENANCE_TODO: Pretty-print error messages with source context.
      const messagesLog = compilationInfo.messages.
      map((m) => `${m.lineNum}:${m.linePos}: ${m.type}: ${m.message}`).
      join('\n');
      error.extra.compilationInfo = compilationInfo;

      if (compilationInfo.messages.some((m) => m.type === 'warning')) {
        if (expectWarning) {
          error.message = `No 'warning' message as expected.\n` + messagesLog;
          this.rec.debug(error);
        } else {
          error.message = `Missing expected compilationInfo 'warning' message.\n` + messagesLog;
          this.rec.validationFailed(error);
        }
      } else {
        if (expectWarning) {
          error.message = `Missing expected 'warning' message.\n` + messagesLog;
          this.rec.validationFailed(error);
        } else {
          error.message = `Found a 'warning' message as expected.\n` + messagesLog;
          this.rec.debug(error);
        }
      }
    });
  }

  /**
   * Add a test expectation for whether a createComputePipeline call succeeds or not.
   */
  expectPipelineResult(args)








  {
    const phonies = [];

    if (args.constants !== undefined) {
      phonies.push(...keysOf(args.constants).map((c) => `_ = ${c};`));
    }
    if (args.reference !== undefined) {
      phonies.push(...args.reference.map((c) => `_ = ${c};`));
    }

    const code =
    args.code +
    `
@compute @workgroup_size(1)
fn main() {
  ${phonies.join('\n')}
}`;

    let shaderModule;
    this.expectGPUError(
      'validation',
      () => {
        shaderModule = this.device.createShaderModule({ code });
      },
      false
    );

    this.expectGPUError(
      'validation',
      () => {
        this.device.createComputePipeline({
          layout: 'auto',
          compute: { module: shaderModule, entryPoint: 'main', constants: args.constants }
        });
      },
      !args.expectedResult
    );
  }

  /**
   * Wraps the code fragment into an entry point.
   *
   * @example
   * ```ts
   * t.wrapInEntryPoint(`var i = 0;`);
   * ```
   */
  wrapInEntryPoint(code, enabledExtensions = []) {
    const enableDirectives = enabledExtensions.map((x) => `enable ${x};`).join('\n      ');

    return `
      ${enableDirectives}

      @compute @workgroup_size(1)
      fn main() {
        ${code}
      }`;
  }
}