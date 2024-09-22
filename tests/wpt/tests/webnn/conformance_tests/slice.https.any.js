// META: title=test WebNN API slice operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-slice
// Produce a slice of the input tensor.
//
// MLOperand slice(
//     MLOperand input, sequence<[EnforceRange] unsigned long>starts,
//     sequence<[EnforceRange] unsigned long>sizes);


const getSlicePrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 0, float16: 0};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const sliceTests = [
  {
    'name': 'slice float32 1D constant tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [12]}, {'sizes': [12]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            0.5424534678459167, 80.44634246826172, 28.32765007019043,
            74.02619171142578, -74.54559326171875, -27.306041717529297,
            -70.42774200439453, 59.82632064819336, -58.46095275878906,
            79.80570983886719, -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 1D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [12]}, {'sizes': [12]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            0.5424534678459167, 80.44634246826172, 28.32765007019043,
            74.02619171142578, -74.54559326171875, -27.306041717529297,
            -70.42774200439453, 59.82632064819336, -58.46095275878906,
            79.80570983886719, -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 2D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [2, 2]}, {'sizes': [2, 4]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            28.32765007019043, 74.02619171142578, -74.54559326171875,
            -27.306041717529297, -58.46095275878906, 79.80570983886719,
            -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 3D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [4, 3, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 1, 1]}, {'sizes': [3, 2, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            57.4397087097168, -4.513182163238525, 74.02619171142578,
            -27.306041717529297, 79.80570983886719, 42.665199279785156
          ],
          'descriptor': {shape: [3, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 4D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 2, 3, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 0, 2, 1]},
          {'sizes': [1, 2, 1, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-27.306041717529297, 42.665199279785156],
          'descriptor': {shape: [1, 2, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 5D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 2, 3, 2, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 0, 2, 1, 0]},
          {'sizes': [1, 2, 1, 1, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-27.306041717529297, 42.665199279785156],
          'descriptor': {shape: [1, 2, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  sliceTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getSlicePrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
