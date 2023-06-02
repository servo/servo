/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { basicExpressionBuilder } from '../../expression.js'; /* @returns a ShaderBuilder that calls the builtin with the given name */
export function builtin(name) {
  return basicExpressionBuilder(values => `${name}(${values.join(', ')})`);
}
