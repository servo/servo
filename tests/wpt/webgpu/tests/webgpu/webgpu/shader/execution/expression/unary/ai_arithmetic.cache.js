/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { abstractInt } from '../../../../util/conversion.js';import { fullI64Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/ai_arithmetic', {
  negation: () => {
    return fullI64Range().map((e) => {
      return { input: abstractInt(e), expected: abstractInt(-e) };
    });
  }
});