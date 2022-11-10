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
      NavigatorML: ['navigator'],
      ML: ['navigator.ml'],
      MLContext: ['context'],
      MLOperand: ['input', 'filter', 'output'],
      MLOperator: ['relu'],
      MLGraphBuilder: ['builder'],
      MLGraph: ['graph']
    });

    for (const executionType of ExecutionArray) {
      const isSync = executionType === 'sync';
      if (self.GLOBAL.isWindow() && isSync) {
        continue;
      }

      for (const deviceType of DeviceTypeArray) {
        const context = navigator.ml.createContext({deviceType});
        // Per spec navigator.ml.createContext should return a MLContext, but
        // in Chromium it returns a Promise<MLContext>. Fail the setup if this
        // happens, since the tests wouldn't make sense.
        if (context instanceof Promise) {
          context.catch(() => {});
          assert_unreached('navigator.ml.createContext returned a Promise');
        }
        self.context = context;
        self.builder = new MLGraphBuilder(context);
        self.input = builder.input('input', {type: 'float32', dimensions: [1, 1, 5, 5]});
        self.filter = builder.constant({type: 'float32', dimensions: [1, 1, 3, 3]}, new Float32Array(9).fill(1));
        self.relu = builder.relu();
        self.output = builder.conv2d(input, filter, {activation: relu, inputLayout: "nchw"});

        if (isSync) {
          self.graph = builder.build({output});
        } else {
          self.graph = await builder.buildAsync({output});
        }
      }
    }
  }
);
