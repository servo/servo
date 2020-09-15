"use strict";

// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

async function loadChromiumResources() {
  const chromiumResources = [
    '/gen/ui/gfx/mojom/color_space.mojom.js',
    '/gen/ui/gfx/mojom/buffer_types.mojom.js',
    '/gen/ui/gfx/mojom/display_color_spaces.mojom.js',
    '/gen/ui/gfx/geometry/mojom/geometry.mojom.js',
    '/gen/ui/display/mojom/display.mojom.js',
    '/gen/third_party/blink/public/mojom/screen_enumeration/screen_enumeration.mojom.js'
  ];

  await loadMojoResources(chromiumResources);
  await loadScript('/resources/testdriver.js');
  await loadScript('/resources/testdriver-vendor.js');
  await loadScript('/resources/chromium/mock-screenenumeration.js');
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
  let newDisplay = new display.mojom.Display({id: id,
                          bounds: new gfx.mojom.Rect({x: bounds.x, y: bounds.y,
                                                      width: bounds.width,
                                                      height: bounds.height}),
                          sizeInPixels: new gfx.mojom.Size({width: bounds.width,
                                                            height: bounds.height}),
                          maximumCursorSize: new gfx.mojom.Size({width: 20, height: 20}),
                          workArea: new gfx.mojom.Rect({x: work_area.x, y: work_area.y,
                                                        width: work_area.width,
                                                        height: work_area.height}),
                          deviceScaleFactor: scale_factor,
                          rotation: display.mojom.Rotation.VALUE_0,
                          touchSupport: display.mojom.TouchSupport.UNAVAILABLE,
                          accelerometerSupport: display.mojom.AccelerometerSupport.UNAVAILABLE,
                          colorSpaces: new gfx.mojom.DisplayColorSpaces({colorSpaces: myColorSpace,
                                                                         bufferFormats: myBufferFormat,
                                                                         sdrWhiteLevel: 1.0}),
                          colorDepth: 10,
                          depthPerComponent: 10,
                          isMonochrome: true,
                          displayFrequency: 120});
  return newDisplay;
}

// Function to construct color space vector.
// Values are purely random but mandatory.
function fillColorSpaceVector() {
  let colorSpaceVector = [];
  for (let i = 0; i < 6; i++) {
    let colorSpace = new gfx.mojom.ColorSpace({
                       primaries: gfx.mojom.ColorSpacePrimaryID.BT709,
                       transfer: gfx.mojom.ColorSpaceTransferID.BT709,
                       matrix: gfx.mojom.ColorSpaceMatrixID.BT709,
                       range: gfx.mojom.ColorSpaceRangeID.LIMITED,
                       customPrimaryMatrix: fillCustomPrimaryMatrix(),
                       transferParams: fillTransferParams()});
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

  let bufferFormat = [gfx.mojom.BufferFormat.RGBA_8888,
                      gfx.mojom.BufferFormat.RGBA_8888,
                      gfx.mojom.BufferFormat.RGBA_8888,
                      gfx.mojom.BufferFormat.RGBA_8888,
                      gfx.mojom.BufferFormat.RGBA_8888,
                      gfx.mojom.BufferFormat.RGBA_8888];
  return bufferFormat;
}
