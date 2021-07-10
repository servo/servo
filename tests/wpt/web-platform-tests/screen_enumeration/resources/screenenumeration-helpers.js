"use strict";

// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

async function loadChromiumResources() {
  await loadScript('/resources/testdriver.js');
  await loadScript('/resources/testdriver-vendor.js');
  const {HelperTypes} = await import('/resources/chromium/mock-screenenumeration.js');
  self.HelperTypes = HelperTypes;
}

async function initialize_screen_enumeration_tests() {
  if (typeof ScreenEnumerationTest === "undefined") {
    const script = document.createElement('script');
    script.src = '/resources/test-only-api.js';
    script.async = false;
    const p = new Promise((resolve, reject) => {
      script.onload = () => { resolve(); };
      script.onerror = e => { reject(e); };
    });
    document.head.appendChild(script);
    await p;

    if (isChromiumBased) {
      await loadChromiumResources();
    }
  }
  assert_implements(ScreenEnumerationTest,
                    'Screen Enumeration testing interface is not available.');
  let enumTest = new ScreenEnumerationTest();
  await enumTest.initialize();
  return enumTest;
}

function screen_enumeration_test(func, name, properties) {
  promise_test(async t => {
    let enumTest = await initialize_screen_enumeration_tests();
    t.add_cleanup(enumTest.reset);
    await func(t, enumTest.getMockScreenEnumeration());
  }, name, properties);
}

// Construct a mock display with provided properties
function makeDisplay(id, bounds, work_area, scale_factor) {
  let myColorSpace = fillColorSpaceVector();
  let myBufferFormat = fillBufferFormatVector();
  return {
    id,
    bounds,
    sizeInPixels: {width: bounds.width, height: bounds.height},
    maximumCursorSize: {width: 20, height: 20},
    workArea: work_area,
    deviceScaleFactor: scale_factor,
    rotation: HelperTypes.Rotation.VALUE_0,
    touchSupport: HelperTypes.TouchSupport.UNAVAILABLE,
    accelerometerSupport: HelperTypes.AccelerometerSupport.UNAVAILABLE,
    colorSpaces: {colorSpaces: myColorSpace,
                  bufferFormats: myBufferFormat,
                  sdrWhiteLevel: 1.0},
    colorDepth: 10,
    depthPerComponent: 10,
    isMonochrome: true,
    displayFrequency: 120
  };
}

// Function to construct color space vector.
// Values are purely random but mandatory.
function fillColorSpaceVector() {
  let colorSpaceVector = [];
  for (let i = 0; i < 6; i++) {
    let colorSpace = {
      primaries: HelperTypes.ColorSpacePrimaryID.BT709,
      transfer: HelperTypes.ColorSpaceTransferID.BT709,
      matrix: HelperTypes.ColorSpaceMatrixID.BT709,
      range: HelperTypes.ColorSpaceRangeID.LIMITED,
      customPrimaryMatrix: fillCustomPrimaryMatrix(),
      transferParams: fillTransferParams()
    };
    colorSpaceVector.push(colorSpace);
  }
  return colorSpaceVector;
}

function fillCustomPrimaryMatrix () {
  let matrix = [1.1, 1.2, 1.3,
                2.1, 2.2, 2.3,
                3.1, 3.2, 3.3];
  return matrix;
}

function fillTransferParams () {
  let params = [1.1, 1.2, 1.3,
                2.1, 2.2, 2.3,
                3.1];
  return params;
}

// Function to construct buffer format vector.
// Values are purely random but mandatory.
function fillBufferFormatVector() {
  const BufferFormat = HelperTypes.BufferFormat;
  let bufferFormat = [BufferFormat.RGBA_8888,
                      BufferFormat.RGBA_8888,
                      BufferFormat.RGBA_8888,
                      BufferFormat.RGBA_8888,
                      BufferFormat.RGBA_8888,
                      BufferFormat.RGBA_8888];
  return bufferFormat;
}
