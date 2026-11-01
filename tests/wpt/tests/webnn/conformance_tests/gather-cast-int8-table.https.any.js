// META: title=test WebNN API gather and cast of an int8 table
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-gather
// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-cast
//
// An int8 lookup table of shape [8, 4] holding -128, -1, 0 and 127 is read
// with runtime int32 indices at the first, middle and last rows (one
// repeated), then widened to float32 in both orders (gather -> cast and
// cast -> gather), optionally scaled by the exact power of two 2^-7, reshaped,
// or read by two gathers sharing the same constant (fanout). The table is a
// constant, and also a runtime input for both orders: the cast tests elsewhere
// widen only non-negative runtime int8 values, so these are the cases where a
// runtime int8 operand read as unsigned (-128 as 128) shows. Every value is an
// integer in [-128, 127] or such an integer times 2^-7, so every transport and
// the scaling are exact; the expected outputs are the source table values.
// Tolerance comes from utils.js: gather, cast and reshape contribute 0 ULP and
// mul contributes its upstream 1 ULP entry.

const int8Table = [
  -128,   -1,    0,  127,
     1,   -2,    3,   -4,
    64,  -64,   32,  -32,
   127, -128,  127, -128,
     5,    6,    7,    8,
  -100,  100,  -50,   50,
  -127,  126, -126,  125,
   127,    0,   -1, -128
];

const indicesA = [0, 4, 7, 4];
const indicesB = [7, 3, 0, 1];

// Rows 0, 4, 7, 4 of the table as float32.
const gatheredRowsA = [
  -128.0, -1.0, 0.0, 127.0, 5.0, 6.0, 7.0, 8.0, 127.0, 0.0, -1.0, -128.0, 5.0,
  6.0, 7.0, 8.0
];
// Rows 7, 3, 0, 1 of the table as float32.
const gatheredRowsB = [
  127.0, 0.0, -1.0, -128.0, 127.0, -128.0, 127.0, -128.0, -128.0, -1.0, 0.0,
  127.0, 1.0, -2.0, 3.0, -4.0
];
// gatheredRowsA * 2^-7, exact.
const gatheredRowsAScaled = [
  -1.0, -0.0078125, 0.0, 0.9921875, 0.0390625, 0.046875, 0.0546875, 0.0625,
  0.9921875, 0.0, -0.0078125, -1.0, 0.0390625, 0.046875, 0.0546875, 0.0625
];

const gatherCastInt8TableTests = [
  {
    'name': 'gather int8 constant table then cast to float32',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'},
          'constant': true
        },
        'indices': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        }
      },
      'operators': [
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indices'}],
          'outputs': 'gathered'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gathered'}, {'type': 'float32'}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': gatheredRowsA,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast int8 constant table to float32 then gather',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'},
          'constant': true
        },
        'indices': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        }
      },
      'operators': [
        {
          'name': 'cast',
          'arguments': [{'input': 'table'}, {'type': 'float32'}],
          'outputs': 'tableFloat32'
        },
        {
          'name': 'gather',
          'arguments': [{'input': 'tableFloat32'}, {'indices': 'indices'}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': gatheredRowsA,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gather int8 runtime table then cast to float32',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'}
        },
        'indices': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        }
      },
      'operators': [
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indices'}],
          'outputs': 'gathered'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gathered'}, {'type': 'float32'}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': gatheredRowsA,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast int8 runtime table to float32 then gather',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'}
        },
        'indices': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        }
      },
      'operators': [
        {
          'name': 'cast',
          'arguments': [{'input': 'table'}, {'type': 'float32'}],
          'outputs': 'tableFloat32'
        },
        {
          'name': 'gather',
          'arguments': [{'input': 'tableFloat32'}, {'indices': 'indices'}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': gatheredRowsA,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gather int8 constant table then cast to float32 then mul by 2^-7',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'},
          'constant': true
        },
        'indices': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        },
        'scale': {
          'data': [0.0078125],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indices'}],
          'outputs': 'gathered'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gathered'}, {'type': 'float32'}],
          'outputs': 'gatheredFloat32'
        },
        {
          'name': 'mul',
          'arguments': [{'a': 'gatheredFloat32'}, {'b': 'scale'}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': gatheredRowsAScaled,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gather int8 constant table then cast to float32 then reshape',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'},
          'constant': true
        },
        'indices': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        }
      },
      'operators': [
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indices'}],
          'outputs': 'gathered'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gathered'}, {'type': 'float32'}],
          'outputs': 'gatheredFloat32'
        },
        {
          'name': 'reshape',
          'arguments': [{'input': 'gatheredFloat32'}, {'newShape': [16]}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': gatheredRowsA,
          'descriptor': {shape: [16], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gather int8 constant table 0D int32 indices then cast to float32',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'},
          'constant': true
        },
        'indices': {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      },
      'operators': [
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indices'}],
          'outputs': 'gathered'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gathered'}, {'type': 'float32'}],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [127.0, 0.0, -1.0, -128.0],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gather int8 constant table fanout to two gathers then cast to float32',
    'graph': {
      'inputs': {
        'table': {
          'data': int8Table,
          'descriptor': {shape: [8, 4], dataType: 'int8'},
          'constant': true
        },
        'indicesA': {
          'data': indicesA,
          'descriptor': {shape: [4], dataType: 'int32'}
        },
        'indicesB': {
          'data': indicesB,
          'descriptor': {shape: [4], dataType: 'int32'}
        }
      },
      'operators': [
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indicesA'}],
          'outputs': 'gatheredA'
        },
        {
          'name': 'gather',
          'arguments': [{'input': 'table'}, {'indices': 'indicesB'}],
          'outputs': 'gatheredB'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gatheredA'}, {'type': 'float32'}],
          'outputs': 'outputA'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'gatheredB'}, {'type': 'float32'}],
          'outputs': 'outputB'
        }
      ],
      'expectedOutputs': {
        'outputA': {
          'data': gatheredRowsA,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        },
        'outputB': {
          'data': gatheredRowsB,
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  }
];

webnn_conformance_test(
    gatherCastInt8TableTests, buildAndExecuteGraph, getPrecisionTolerance);
