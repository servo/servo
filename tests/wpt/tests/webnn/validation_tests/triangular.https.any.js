// META: title=validation tests for WebNN API triangular operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

promise_test(async t => {
  const builder = new MLGraphBuilder(context);

  // The input tensor which is at least 2-D.
  for (let shape of allWebNNShapesArray.slice(0, 2)) {
    for (let dataType of allWebNNOperandDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        continue;
      }
      const input = builder.input(`input${++inputIndex}`, {dataType, shape});
      const label = 'triangular_3';
      const options = {label};
      const regrexp = new RegExp('\\[' + label + '\\]');
      assert_throws_with_label(
          () => builder.triangular(input, options), regrexp);
    }
  }
}, '[triangular] TypeError is expected if input\'s rank is less than 2');

validateInputFromAnotherBuilder('triangular');
