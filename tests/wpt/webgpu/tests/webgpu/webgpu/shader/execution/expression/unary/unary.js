/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { basicExpressionBuilder } from '../expression.js'; /* @returns a ShaderBuilder that evaluates a prefix unary operation */
export function unary(op) {
  return basicExpressionBuilder(value => `${op}(${value})`);
}

export function assignment() {
  return basicExpressionBuilder(value => `${value}`);
}
