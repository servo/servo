// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reducemax
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceMax(MLOperand input, optional MLReduceOptions options = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: 0,
  };
};

const reduceMaxTests = [
  {
    'name': 'reduceMax float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [32.16658401489258],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 32.16658401489258,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [32.16658401489258],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 32.16658401489258,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 1D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 1D tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [{'input': 'reduceMaxInput'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': [
            90.42288208007812, 75.90379333496094, 94.99645233154297,
            96.55397033691406, 99.77313232421875, 20.253753662109375
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments':
            [{'input': 'reduceMaxInput'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': [
            62.504642486572266, 96.55397033691406, 99.77313232421875,
            -21.557384490966797, 94.99645233154297, 37.28493118286133
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [
          {'input': 'reduceMaxInput'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [
          {'input': 'reduceMaxInput'}, {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': [99.77313232421875],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [
          {'input': 'reduceMaxInput'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': 99.77313232421875,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceMax float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [
          {'input': 'reduceMaxInput'}, {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': [99.77313232421875],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceMax float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [
          {'input': 'reduceMaxInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': [
            90.42288208007812, 94.99645233154297, 96.55397033691406,
            99.77313232421875
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceMax float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceMaxInput': {
          'data': [
            32.16658401489258,   90.42288208007812,  -26.341794967651367,
            -7.147959232330322,  75.90379333496094,  -48.2042121887207,
            -53.09425354003906,  66.66099548339844,  -96.16854095458984,
            -88.30545043945312,  94.99645233154297,  37.28493118286133,
            -42.209861755371094, 96.55397033691406,  0.8807229995727539,
            62.504642486572266,  36.650634765625,    99.77313232421875,
            -72.86485290527344,  -46.03200912475586, 20.253753662109375,
            -21.557384490966797, -51.28727340698242, -42.58832931518555
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceMax',
        'arguments': [
          {'input': 'reduceMaxInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceMaxOutput'
      }],
      'expectedOutputs': {
        'reduceMaxOutput': {
          'data': [
            90.42288208007812, 94.99645233154297, 96.55397033691406,
            99.77313232421875
          ],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
];

if (navigator.ml) {
  reduceMaxTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
