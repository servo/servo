// META: title=validation tests for WebNN API element-wise binary operations
// META: global=window
// META: variant=?op=add&device=cpu
// META: variant=?op=add&device=gpu
// META: variant=?op=add&device=npu
// META: variant=?op=sub&device=cpu
// META: variant=?op=sub&device=gpu
// META: variant=?op=sub&device=npu
// META: variant=?op=mul&device=cpu
// META: variant=?op=mul&device=gpu
// META: variant=?op=mul&device=npu
// META: variant=?op=div&device=cpu
// META: variant=?op=div&device=gpu
// META: variant=?op=div&device=npu
// META: variant=?op=max&device=cpu
// META: variant=?op=max&device=gpu
// META: variant=?op=max&device=npu
// META: variant=?op=min&device=cpu
// META: variant=?op=min&device=gpu
// META: variant=?op=min&device=npu
// META: variant=?op=pow&device=cpu
// META: variant=?op=pow&device=gpu
// META: variant=?op=pow&device=npu
// META: script=../resources/utils_validation.js

'use strict';

const queryParams = new URLSearchParams(window.location.search);
const operatorName = queryParams.get('op');

const label = 'elementwise_binary_op';
const regrexp = new RegExp('\\[' + label + '\\]');
const tests = [
  {
    name: '[binary] Test bidirectionally broadcastable dimensions.',
    //  Both inputs have axes of length one which are expanded
    //  during broadcasting.
    a: {dataType: 'float32', shape: [8, 1, 6, 1]},
    b: {dataType: 'float32', shape: [7, 1, 5]},
    output: {dataType: 'float32', shape: [8, 7, 6, 5]}
  },
  {
    name: '[binary] Test unidirectionally broadcastable dimensions.',
    // Input a has a single axis of length one which is
    // expanded during broadcasting.
    a: {dataType: 'float32', shape: [4, 2, 1]},
    b: {dataType: 'float32', shape: [4]},
    output: {dataType: 'float32', shape: [4, 2, 4]}
  },
  {
    name: '[binary] Test scalar broadcasting.',
    a: {dataType: 'float32', shape: [4, 2, 4]},
    b: {dataType: 'float32', shape: []},
    output: {dataType: 'float32', shape: [4, 2, 4]}
  },
  {
    name: '[binary] Throw if the input shapes are not broadcastable.',
    a: {dataType: 'float32', shape: [4, 2]},
    b: {dataType: 'float32', shape: [4]},
  },
  {
    name: '[binary] Throw if the input types don\'t match.',
    a: {dataType: 'float32', shape: [4, 2]},
    b: {dataType: 'int32', shape: [1]},
  },
];

tests.forEach(test => {
  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    if (!context.opSupportLimits().input.dataTypes.includes(
            test.a.dataType)) {
      assert_throws_js(TypeError, () => builder.input('a', test.a));
      return;
    }
    if (!context.opSupportLimits().input.dataTypes.includes(
            test.b.dataType)) {
      assert_throws_js(TypeError, () => builder.input('b', test.b));
      return;
    }
    const a = builder.input('a', test.a);
    const b = builder.input('b', test.b);

    if (test.output) {
      const output = builder[operatorName](a, b);
      assert_equals(output.dataType, test.output.dataType);
      assert_array_equals(output.shape, test.output.shape);
    } else {
      const options = {label};
      assert_throws_with_label(
          () => builder[operatorName](a, b, options), regrexp);
    }
  }, test.name.replace('[binary]', `[${operatorName}]`));
});

validateTwoInputsOfSameDataType(operatorName, label);
validateTwoInputsBroadcastable(operatorName, label);
validateTwoInputsFromMultipleBuilders(operatorName);
validateTwoBroadcastableInputsTensorLimit(operatorName, label);
