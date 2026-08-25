// META: title=test WebNN API tile operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://github.com/webmachinelearning/webnn/issues/375
// Represents the tile operation that repeats a tensor the given number of
// times along each axis.
//
// MLOperand tile(
//     MLOperand input, sequence<unsigned long> repetitions, optional
//     MLOperatorOptions options = {});

const tileTests = [
  {
    'name': 'tile float32 0D scalar tensor by repetitions=[]',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [0.5],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': []}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [0.5],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'tile float32 1D constant tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {shape: [4], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [1, 2, 3, 4, 1, 2, 3, 4],
          'descriptor': {shape: [8], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'tile float32 1D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {shape: [4], dataType: 'float32'},
          'constant': false
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [1, 2, 3, 4, 1, 2, 3, 4],
          'descriptor': {shape: [8], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'tile float16 1D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {shape: [4], dataType: 'float16'},
          'constant': false
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [1, 2, 3, 4, 1, 2, 3, 4],
          'descriptor': {shape: [8], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'tile float16 2D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {shape: [2, 2], dataType: 'float16'},
          'constant': false
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2, 3]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [
            1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4,
            1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'tile uint32 2D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {shape: [2, 2], dataType: 'uint32'},
          'constant': false
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2, 3]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [
            1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4,
            1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4
          ],
          'descriptor': {shape: [4, 6], dataType: 'uint32'}
        }
      }
    }
  },
  {
    'name': 'tile int32 4D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'int32'},
          'constant': false
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [1, 1, 2, 2]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [1, 2, 1, 2, 3, 4, 3, 4, 1, 2, 1, 2, 3, 4, 3, 4],
          'descriptor': {shape: [1, 1, 4, 4], dataType: 'int32'}
        }
      }
    }
  },
];

webnn_conformance_test(tileTests, buildAndExecuteGraph, getZeroULPTolerance);
