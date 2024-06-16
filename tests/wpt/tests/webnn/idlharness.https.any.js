// META: global=window,dedicatedworker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=./resources/utils.js
// META: timeout=long

// https://webmachinelearning.github.io/webnn/

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
      MLActivation: ['relu'],
      MLGraphBuilder: ['builder'],
      MLGraph: ['graph']
    });

    self.context = await navigator.ml.createContext();
    self.builder = new MLGraphBuilder(self.context);
    self.input =
        builder.input('input', {dataType: 'float32', dimensions: [2, 3]});
    self.constant = builder.constant(
        {dataType: 'float32', dimensions: [2, 3]},
        new Float32Array(2 * 3).fill(1));

    // Create an activation which won't be used in the graph.
    self.relu = builder.relu();

    self.output = builder.add(input, constant);

    self.graph = await builder.build({output});
  }
);
