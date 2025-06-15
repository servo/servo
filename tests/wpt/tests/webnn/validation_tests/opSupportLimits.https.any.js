// META: title=validation context.opSupportLimits() interface
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';


const tests = [
  {
    operator: 'logicalAnd',
    limits: {
      a: {dataTypes: ['uint8']},
      b: {dataTypes: ['uint8']},
      output: {dataTypes: ['uint8']},
    }
  },
  {
    operator: 'logicalOr',
    limits: {
      a: {dataTypes: ['uint8']},
      b: {dataTypes: ['uint8']},
      output: {dataTypes: ['uint8']},
    }
  },
  {
    operator: 'logicalXor',
    limits: {
      a: {dataTypes: ['uint8']},
      b: {dataTypes: ['uint8']},
      output: {dataTypes: ['uint8']},
    }
  },
  {
    operator: 'logicalNot',
    limits: {
      a: {dataTypes: ['uint8']},
      output: {dataTypes: ['uint8']},
    }
  }
];

tests.forEach(test => promise_test(async t => {
                const limits = context.opSupportLimits()[test.operator];
                for (let [name, expected] of Object.entries(test.limits)) {
                  for (let actualDataType of limits[name].dataTypes) {
                    assert_in_array(
                        actualDataType, expected.dataTypes,
                        `${test.operator}.${name}.dataTypes`);
                  }
                }
              }, `check opSupportLimits data types of ${test.operator}`));
