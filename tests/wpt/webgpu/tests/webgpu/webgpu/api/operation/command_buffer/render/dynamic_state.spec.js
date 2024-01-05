/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests of the behavior of the viewport/scissor/blend/reference states.

TODO:
- {viewport, scissor rect, blend color, stencil reference}:
  Test rendering result with {various values}.
    - Set the state in different ways to make sure it gets the correct value in the end: {
        - state unset (= default)
        - state explicitly set once to {default value, another value}
        - persistence: [set, draw, draw] (fn should differentiate from [set, draw] + [draw])
        - overwriting: [set(1), draw, set(2), draw] (fn should differentiate from [set(1), set(2), draw, draw])
        - overwriting: [set(1), set(2), draw] (fn should differentiate from [set(1), draw] but not [set(2), draw])
        - }
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);