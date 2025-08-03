// META: title=test WebNN API pad operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-pad
// Inflate the tensor with constant or mirrored values on the edges.
//
// enum MLPaddingMode {
//   "constant",
//   "edge",
//   "reflection"
// };
//
// dictionary MLPadOptions {
//   MLPaddingMode mode = "constant";
//   MLNumber value = 0;
// };
//
// MLOperand pad(
//     MLOperand input, sequence<[EnforceRange] unsigned long>beginningPadding,
//     sequence<[EnforceRange] unsigned long>endingPadding,
//     optional MLPadOptions options = {});

const padTests = [
  {
    'name': 'pad float32 1D constant tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [9], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1]},
          {'endingPadding': [1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547, 0
          ],
          'descriptor': {shape: [11], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 1D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [9], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1]},
          {'endingPadding': [1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547, 0
          ],
          'descriptor': {shape: [11], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 2D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,
            0,
            0,
            0,
            0,
            0,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            0,
            0,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            0,
            0,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            0,
            0,
            0,
            0,
            0,
            0
          ],
          'descriptor': {shape: [5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 3D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [1, 3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1, 1]},
          {'endingPadding': [1, 1, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            0,
            0,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            0,
            0,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0
          ],
          'descriptor': {shape: [3, 5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 4D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 1, 1, 1]},
          {'endingPadding': [0, 1, 1, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            22.76361846923828,
            0,
            0,
            -21.168529510498047,
            0,
            0,
            -91.66168975830078,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            16.863798141479492,
            0,
            0,
            60.51472091674805,
            0,
            0,
            -70.56755065917969,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -60.643272399902344,
            0,
            0,
            -47.8821907043457,
            0,
            0,
            68.72557830810547,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0
          ],
          'descriptor': {shape: [1, 5, 5, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 5D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [1, 3, 3, 1, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 1, 1, 0, 1]},
          {'endingPadding': [0, 1, 1, 0, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            22.76361846923828,
            0,
            0,
            -21.168529510498047,
            0,
            0,
            -91.66168975830078,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            16.863798141479492,
            0,
            0,
            60.51472091674805,
            0,
            0,
            -70.56755065917969,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -60.643272399902344,
            0,
            0,
            -47.8821907043457,
            0,
            0,
            68.72557830810547,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0
          ],
          'descriptor': {shape: [1, 5, 5, 1, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 2D tensor explicit options.mode=\'constant\'',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'mode': 'constant'}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,
            0,
            0,
            0,
            0,
            0,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            0,
            0,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            0,
            0,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            0,
            0,
            0,
            0,
            0,
            0
          ],
          'descriptor': {shape: [5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 2D tensor options.value default constant mode',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'value': 1}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            1,
            1,
            1,
            1,
            1,
            1,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            1,
            1,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            1,
            1,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            1,
            1,
            1,
            1,
            1,
            1
          ],
          'descriptor': {shape: [5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 2D tensor with options.value as NaN',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'value': NaN}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,
            0,
            0,
            0,
            0,
            0,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            0,
            0,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            0,
            0,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            0,
            0,
            0,
            0,
            0,
            0
          ],
          'descriptor': {shape: [5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 2D tensor with options.value as Infinity',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'value': Infinity}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            Infinity,
            Infinity,
            Infinity,
            Infinity,
            Infinity,
            Infinity,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            Infinity,
            Infinity,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            Infinity,
            Infinity,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            Infinity,
            Infinity,
            Infinity,
            Infinity,
            Infinity,
            Infinity
          ],
          'descriptor': {shape: [5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad int64 2D tensor with options.value as bigint',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22, -21, -91,
            16, 60, -70,
            -60, -47, 68
          ],
          'descriptor': {shape: [3, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'value': 9007199254740992n}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            22,
            -21,
            -91,
            9007199254740992n,
            9007199254740992n,
            16,
            60,
            -70,
            9007199254740992n,
            9007199254740992n,
            -60,
            -47,
            68,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n,
            9007199254740992n
          ],
          'descriptor': {shape: [5, 5], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'pad float32 2D tensor with options.value as -Infinity',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'value': -Infinity}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            -Infinity,
            -Infinity,
            -Infinity,
            -Infinity,
            -Infinity,
            -Infinity,
            22.76361846923828,
            -21.168529510498047,
            -91.66168975830078,
            -Infinity,
            -Infinity,
            16.863798141479492,
            60.51472091674805,
            -70.56755065917969,
            -Infinity,
            -Infinity,
            -60.643272399902344,
            -47.8821907043457,
            68.72557830810547,
            -Infinity,
            -Infinity,
            -Infinity,
            -Infinity,
            -Infinity,
            -Infinity
          ],
          'descriptor': {shape: [5, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 4D tensor options.mode=\'edge\'',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 2, 2, 0]},
          {'endingPadding': [0, 2, 2, 0]}, {'options': {'mode': 'edge'}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            22.76361846923828,   22.76361846923828,   22.76361846923828,
            -21.168529510498047, -91.66168975830078,  -91.66168975830078,
            -91.66168975830078,  22.76361846923828,   22.76361846923828,
            22.76361846923828,   -21.168529510498047, -91.66168975830078,
            -91.66168975830078,  -91.66168975830078,  22.76361846923828,
            22.76361846923828,   22.76361846923828,   -21.168529510498047,
            -91.66168975830078,  -91.66168975830078,  -91.66168975830078,
            16.863798141479492,  16.863798141479492,  16.863798141479492,
            60.51472091674805,   -70.56755065917969,  -70.56755065917969,
            -70.56755065917969,  -60.643272399902344, -60.643272399902344,
            -60.643272399902344, -47.8821907043457,   68.72557830810547,
            68.72557830810547,   68.72557830810547,   -60.643272399902344,
            -60.643272399902344, -60.643272399902344, -47.8821907043457,
            68.72557830810547,   68.72557830810547,   68.72557830810547,
            -60.643272399902344, -60.643272399902344, -60.643272399902344,
            -47.8821907043457,   68.72557830810547,   68.72557830810547,
            68.72557830810547
          ],
          'descriptor': {shape: [1, 7, 7, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'pad float32 4D tensor options.mode=\'reflection\'',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.76361846923828, -21.168529510498047, -91.66168975830078,
            16.863798141479492, 60.51472091674805, -70.56755065917969,
            -60.643272399902344, -47.8821907043457, 68.72557830810547
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 2, 2, 0]},
          {'endingPadding': [0, 2, 2, 0]}, {'options': {'mode': 'reflection'}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            68.72557830810547,   -47.8821907043457,   -60.643272399902344,
            -47.8821907043457,   68.72557830810547,   -47.8821907043457,
            -60.643272399902344, -70.56755065917969,  60.51472091674805,
            16.863798141479492,  60.51472091674805,   -70.56755065917969,
            60.51472091674805,   16.863798141479492,  -91.66168975830078,
            -21.168529510498047, 22.76361846923828,   -21.168529510498047,
            -91.66168975830078,  -21.168529510498047, 22.76361846923828,
            -70.56755065917969,  60.51472091674805,   16.863798141479492,
            60.51472091674805,   -70.56755065917969,  60.51472091674805,
            16.863798141479492,  68.72557830810547,   -47.8821907043457,
            -60.643272399902344, -47.8821907043457,   68.72557830810547,
            -47.8821907043457,   -60.643272399902344, -70.56755065917969,
            60.51472091674805,   16.863798141479492,  60.51472091674805,
            -70.56755065917969,  60.51472091674805,   16.863798141479492,
            -91.66168975830078,  -21.168529510498047, 22.76361846923828,
            -21.168529510498047, -91.66168975830078,  -21.168529510498047,
            22.76361846923828
          ],
          'descriptor': {shape: [1, 7, 7, 1], dataType: 'float32'}
        }
      }
    }
  },


  // float16 tests
  {
    'name': 'pad float16 1D constant tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [9], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1]},
          {'endingPadding': [1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75, 0
          ],
          'descriptor': {shape: [11], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 1D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [9], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1]},
          {'endingPadding': [1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75, 0
          ],
          'descriptor': {shape: [11], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 2D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 0,         0,          0,        0,
            0, 22.765625, -21.171875, -91.6875, 0,
            0, 16.859375, 60.5,       -70.5625, 0,
            0, -60.65625, -47.875,    68.75,    0,
            0, 0,         0,          0,        0
          ],
          'descriptor': {shape: [5, 5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 3D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [1, 3, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1, 1]},
          {'endingPadding': [1, 1, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         22.765625, -21.171875, -91.6875, 0, 0,
            16.859375, 60.5,      -70.5625,   0,        0, -60.65625,
            -47.875,   68.75,     0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0,          0,        0, 0,
            0,         0,         0
          ],
          'descriptor': {shape: [3, 5, 5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 4D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 1, 1, 1]},
          {'endingPadding': [0, 1, 1, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 0,         0, 0, 0,        0, 0, 0,         0, 0, 0,          0,
            0, 0,         0, 0, 0,        0, 0, 22.765625, 0, 0, -21.171875, 0,
            0, -91.6875,  0, 0, 0,        0, 0, 0,         0, 0, 16.859375,  0,
            0, 60.5,      0, 0, -70.5625, 0, 0, 0,         0, 0, 0,          0,
            0, -60.65625, 0, 0, -47.875,  0, 0, 68.75,     0, 0, 0,          0,
            0, 0,         0, 0, 0,        0, 0, 0,         0, 0, 0,          0,
            0, 0,         0
          ],
          'descriptor': {shape: [1, 5, 5, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 5D tensor default options',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [1, 3, 3, 1, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 1, 1, 0, 1]},
          {'endingPadding': [0, 1, 1, 0, 1]}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 0,         0, 0, 0,        0, 0, 0,         0, 0, 0,          0,
            0, 0,         0, 0, 0,        0, 0, 22.765625, 0, 0, -21.171875, 0,
            0, -91.6875,  0, 0, 0,        0, 0, 0,         0, 0, 16.859375,  0,
            0, 60.5,      0, 0, -70.5625, 0, 0, 0,         0, 0, 0,          0,
            0, -60.65625, 0, 0, -47.875,  0, 0, 68.75,     0, 0, 0,          0,
            0, 0,         0, 0, 0,        0, 0, 0,         0, 0, 0,          0,
            0, 0,         0
          ],
          'descriptor': {shape: [1, 5, 5, 1, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 2D tensor explicit options.mode=\'constant\'',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'mode': 'constant'}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            0, 0,         0,          0,        0,
            0, 22.765625, -21.171875, -91.6875, 0,
            0, 16.859375, 60.5,       -70.5625, 0,
            0, -60.65625, -47.875,    68.75,    0,
            0, 0,         0,          0,        0
          ],
          'descriptor': {shape: [5, 5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 2D tensor options.value default constant mode',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [1, 1]},
          {'endingPadding': [1, 1]}, {'options': {'value': 1}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            1, 1,         1,          1,        1,
            1, 22.765625, -21.171875, -91.6875, 1,
            1, 16.859375, 60.5,       -70.5625, 1,
            1, -60.65625, -47.875,    68.75,    1,
            1, 1,         1,          1,        1
          ],
          'descriptor': {shape: [5, 5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 4D tensor options.mode=\'edge\'',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 2, 2, 0]},
          {'endingPadding': [0, 2, 2, 0]}, {'options': {'mode': 'edge'}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            22.765625, 22.765625, 22.765625, -21.171875, -91.6875,   -91.6875,
            -91.6875,  22.765625, 22.765625, 22.765625,  -21.171875, -91.6875,
            -91.6875,  -91.6875,  22.765625, 22.765625,  22.765625,  -21.171875,
            -91.6875,  -91.6875,  -91.6875,  16.859375,  16.859375,  16.859375,
            60.5,      -70.5625,  -70.5625,  -70.5625,   -60.65625,  -60.65625,
            -60.65625, -47.875,   68.75,     68.75,      68.75,      -60.65625,
            -60.65625, -60.65625, -47.875,   68.75,      68.75,      68.75,
            -60.65625, -60.65625, -60.65625, -47.875,    68.75,      68.75,
            68.75
          ],
          'descriptor': {shape: [1, 7, 7, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'pad float16 4D tensor options.mode=\'reflection\'',
    'graph': {
      'inputs': {
        'padInput': {
          'data': [
            22.765625, -21.171875, -91.6875, 16.859375, 60.5, -70.5625,
            -60.65625, -47.875, 68.75
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'pad',
        'arguments': [
          {'input': 'padInput'}, {'beginningPadding': [0, 2, 2, 0]},
          {'endingPadding': [0, 2, 2, 0]}, {'options': {'mode': 'reflection'}}
        ],
        'outputs': 'padOutput'
      }],
      'expectedOutputs': {
        'padOutput': {
          'data': [
            68.75,     -47.875,    -60.65625, -47.875,    68.75,     -47.875,
            -60.65625, -70.5625,   60.5,      16.859375,  60.5,      -70.5625,
            60.5,      16.859375,  -91.6875,  -21.171875, 22.765625, -21.171875,
            -91.6875,  -21.171875, 22.765625, -70.5625,   60.5,      16.859375,
            60.5,      -70.5625,   60.5,      16.859375,  68.75,     -47.875,
            -60.65625, -47.875,    68.75,     -47.875,    -60.65625, -70.5625,
            60.5,      16.859375,  60.5,      -70.5625,   60.5,      16.859375,
            -91.6875,  -21.171875, 22.765625, -21.171875, -91.6875,  -21.171875,
            22.765625
          ],
          'descriptor': {shape: [1, 7, 7, 1], dataType: 'float16'}
        }
      }
    }
  }
];

if (navigator.ml) {
  padTests.filter(isTargetTest).forEach((test) => {
    webnn_conformance_test(buildAndExecuteGraph, getZeroULPTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
