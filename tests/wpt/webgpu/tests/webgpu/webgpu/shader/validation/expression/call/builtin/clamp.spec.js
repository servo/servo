/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ const builtin = 'clamp';
export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  TypeF16,
  elementType,
  kAllFloatAndIntegerScalarsAndVectors,
} from '../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval,
} from './const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kAllFloatAndIntegerScalarsAndVectors);

g.test('values')
  .desc(
    `
Validates that constant evaluation and override evaluation of ${builtin}() rejects invalid values
`
  )
  .params(u =>
    u
      .combine('stage', kConstantAndOverrideStages)
      .combine('type', keysOf(kValuesTypes))
      .filter(u => stageSupportsType(u.stage, kValuesTypes[u.type]))
      .beginSubcases()
      .expand('e', u => fullRangeForType(kValuesTypes[u.type], 3))
      .expand('low', u => fullRangeForType(kValuesTypes[u.type], 4))
      .expand('high', u => fullRangeForType(kValuesTypes[u.type], 4))
  )
  .beforeAllSubcases(t => {
    if (elementType(kValuesTypes[t.params.type]) === TypeF16) {
      t.selectDeviceOrSkipTestCase('shader-f16');
    }
  })
  .fn(t => {
    const type = kValuesTypes[t.params.type];
    const expectedResult = t.params.low <= t.params.high;
    validateConstOrOverrideBuiltinEval(
      t,
      builtin,
      expectedResult,
      [type.create(t.params.e), type.create(t.params.low), type.create(t.params.high)],
      t.params.stage
    );
  });
