// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reducelogsumexp
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceLogSumExp(MLOperand input, optional MLReduceOptions options
// = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: getReducedElementCount(graphResources) * 2 + 18,
  };
};

const reduceLogSumExpTests = [
  {
    'name': 'reduceLogSumExp float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [0.7974132895469666],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 0.7974132895469666,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [0.7974132895469666],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments':
            [{'input': 'reduceLogSumExpInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 0.7974132895469666,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceLogSumExp float32 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            -4.025670051574707,  -9.444348335266113,   -3.1193981170654297,
            -5.943697929382324,  -0.3701804578304291,  -4.397126197814941,
            -6.605968475341797,  -5.534277439117432,   -7.361471176147461,
            -1.9987547397613525, -9.093968391418457,   -8.693618774414062,
            -8.416788101196289,  -1.010741114616394,   -9.814584732055664,
            -9.725259780883789,  -9.157071113586426,   -0.001698818989098072,
            -9.963415145874023,  -5.991659641265869,   -6.180599689483643,
            -1.2336505651474,    -0.44234341382980347, -6.990072250366211
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 1.1666961908340454,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceLogSumExp float32 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            1, 5, 7, 5, 7, 5, 4, 2, 1, 5, 8, 2,
            4, 1, 4, 5, 4, 8, 6, 2, 7, 7, 8, 5
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 9.607237815856934,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceLogSumExp float32 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            -6, -3, -5,  -1,  -9, -5, -1, -2, -10, -1, -5, -7,
            -7, -3, -10, -10, -8, -6, -2, -6, -1,  -9, -5, -2
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 0.7001367211341858,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [{'input': 'reduceLogSumExpInput'}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments':
            [{'input': 'reduceLogSumExpInput'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': [
            8.55212688446045, 3.985233783721924, 5.52872896194458,
            9.081488609313965, 6.996237754821777, 9.759706497192383
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments':
            [{'input': 'reduceLogSumExpInput'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': [
            4.66951847076416, 9.08117961883545, 8.533217430114746,
            9.270560264587402, 6.450263977050781, 8.917200088500977
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [
          {'input': 'reduceLogSumExpInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [
          {'input': 'reduceLogSumExpInput'},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': [10.39477825164795],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [
          {'input': 'reduceLogSumExpInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': 10.39477825164795,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceLogSumExp float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [
          {'input': 'reduceLogSumExpInput'},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': [10.39477825164795],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceLogSumExp float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [
          {'input': 'reduceLogSumExpInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': [
            8.563796997070312, 5.500619411468506, 9.753945350646973,
            9.20864486694336
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceLogSumExp float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceLogSumExpInput': {
          'data': [
            0.7974132895469666,  5.046889781951904,   8.520371437072754,
            1.4063042402267456,  0.11882661283016205, 0.2858544886112213,
            1.9325640201568604,  3.7939958572387695,  2.6040232181549072,
            4.937509536743164,   4.571482181549072,   0.786512017250061,
            0.21018670499324799, 9.063042640686035,   4.099809646606445,
            4.596248626708984,   0.2549232244491577,  1.159480094909668,
            6.802876949310303,   5.234325408935547,   8.914905548095703,
            9.166799545288086,   5.717507362365723,   0.3255050778388977
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceLogSumExp',
        'arguments': [
          {'input': 'reduceLogSumExpInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceLogSumExpOutput'
      }],
      'expectedOutputs': {
        'reduceLogSumExpOutput': {
          'data': [
            8.563796997070312, 5.500619411468506, 9.753945350646973,
            9.20864486694336
          ],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reduceLogSumExpTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
