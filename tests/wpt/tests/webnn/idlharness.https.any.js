// META: global=window,dedicatedworker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=./resources/utils.js
// META: timeout=long

// https://www.w3.org/TR/webnn/

'use strict';

idl_test(
  ['webnn'],
  ['html', 'webidl', 'webgpu'],
  async (idl_array) => {
    if (self.GLOBAL.isWindow()) {
      idl_array.add_objects({ Navigator: ['navigator'] });
    } else if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    }

    idl_array.add_objects({
      ML: ['navigator.ml'],
      MLContext: ['context'],
      MLOperand: ['input', 'constant', 'output'],
      MLGraphBuilder: ['builder'],
      MLGraph: ['graph']
    });

    self.context = await navigator.ml.createContext();
    self.builder = new MLGraphBuilder(self.context);
    self.input = builder.input('input', {dataType: 'float32', shape: [2, 3]});
    self.constant = builder.constant(
        {dataType: 'float32', shape: [2, 3]}, new Float32Array(2 * 3).fill(1));

    self.output = builder.add(input, constant);

    self.graph = await builder.build({output});
  }
);
