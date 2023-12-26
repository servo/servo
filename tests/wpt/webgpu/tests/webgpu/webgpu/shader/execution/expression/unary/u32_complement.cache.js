/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { u32 } from '../../../../util/conversion.js';import { fullU32Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/u32_complement', {
  complement: () => {
    return fullU32Range().map((e) => {
      return { input: u32(e), expected: u32(~e) };
    });
  }
});