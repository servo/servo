/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Flow control tests for interesting complex cases.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('continue_in_switch_in_for_loop')
  .desc('Test flow control for a continue statement in a switch, in a for-loop')
  .params(u => u.combine('preventValueOptimizations', [true, false]))
  .fn(t => {
    runFlowControlTest(
      t,
      f =>
        `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < 3; i++) {
    ${f.expect_order(1, 4, 6)}
    switch (i) {
      case 2: {
        ${f.expect_order(7)}
        break;
      }
      case 1: {
        ${f.expect_order(5)}
        continue;
      }
      default: {
        ${f.expect_order(2)}
        break;
      }
    }
    ${f.expect_order(3, 8)}
  }
  ${f.expect_order(9)}
`
    );
  });
