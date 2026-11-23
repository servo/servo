// spec link: https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-canvas-getcontextattributes
// This test should be kept in sync with getContextAttributes.html (the window version of this test).

importScripts("/resources/testharness.js");

var testScenarios = [
    // defaults
    {testDescription: "Test default offscreen context creation attributes in a worker",
        canvasContextAttributes: {},
        expectedContextAttributes: {
            alpha: true,
            desynchronized: false,
            willReadFrequently: false,
            colorSpace: "srgb",
            colorType: "unorm8",
            toneMapping: {mode: "standard"}}},
    // alpha
    {testDescription: "Test offscreen context creation attributes alpha: true in a worker",
        canvasContextAttributes: {alpha: true},
        expectedContextAttributes: {alpha: true}},
    {testDescription: "Test offscreen context creation attributes alpha: false in a worker",
        canvasContextAttributes: {alpha: false},
        expectedContextAttributes: {alpha: false}},
    // colorSpace
    {testDescription: "Test offscreen context creation attributes colorSpace: 'srgb' in a worker",
        canvasContextAttributes: {colorSpace: "srgb"},
        expectedContextAttributes: {colorSpace: "srgb"}},
    {testDescription: "Test offscreen context creation attributes colorSpace: 'display-p3' in a worker",
        canvasContextAttributes: {colorSpace: "display-p3"},
        expectedContextAttributes: {colorSpace: "display-p3"}},
    {testDescription: "Test offscreen context creation attributes colorSpace: 'srgb-linear' in a worker",
        canvasContextAttributes: {colorSpace: "srgb-linear"},
        expectedContextAttributes: {colorSpace: "srgb-linear"}},
    {testDescription: "Test offscreen context creation attributes colorSpace: 'display-p3-linear' in a worker",
        canvasContextAttributes: {colorSpace: "display-p3-linear"},
        expectedContextAttributes: {colorSpace: "display-p3-linear"}},
    // desynchronized
    {testDescription: "Test offscreen context creation attributes desynchronized: true in a worker",
        canvasContextAttributes: {desynchronized: true},
        expectedContextAttributes: {desynchronized: true}},
    {testDescription: "Test offscreen context creation attributes desynchronized: false in a worker",
        canvasContextAttributes: {desynchronized: false},
        expectedContextAttributes: {desynchronized: false}},
    // willReadFrequently
    {testDescription: "Test offscreen context creation attributes willReadFrequently: true in a worker",
        canvasContextAttributes: {willReadFrequently: true},
        expectedContextAttributes: {willReadFrequently: true}},
    {testDescription: "Test offscreen context creation attributes willReadFrequently: false in a worker",
        canvasContextAttributes: {willReadFrequently: false},
        expectedContextAttributes: {willReadFrequently: false}},
    // colorType
    {testDescription: "Test offscreen context creation attributes colorType: unorm8 in a worker",
        canvasContextAttributes: {colorType: "unorm8"},
        expectedContextAttributes: {colorType: "unorm8"}},
    {testDescription: "Test offscreen context creation attributes colorType: float16 in a worker",
        canvasContextAttributes: {colorType: "float16"},
        expectedContextAttributes: {colorType: "float16"}},
    // toneMapping
    {testDescription: "Test offscreen context creation attributes toneMapping: {} in a worker",
        canvasContextAttributes: {toneMapping: {}},
        expectedContextAttributes: {toneMapping: {mode: "standard"}}},
    {testDescription: "Test offscreen context creation attributes toneMapping: {mode: 'standard'} in a worker",
        canvasContextAttributes: {toneMapping: {mode: "standard"}},
        expectedContextAttributes: {toneMapping: {mode: "standard"}}},
    {testDescription: "Test offscreen context creation attributes toneMapping: {mode: 'extended'} in a worker",
        canvasContextAttributes: {toneMapping: {mode: "extended"}},
        expectedContextAttributes: {toneMapping: {mode: "extended"}}},
];

function runTestScenario(canvas, testScenario) {
    test(function() {
        var ctx = canvas.getContext('2d', testScenario.canvasContextAttributes);
        var contextAttributes = ctx.getContextAttributes();
        var expected = testScenario.expectedContextAttributes;
        if (expected.alpha !== undefined)
            assert_equals(contextAttributes.alpha, expected.alpha);
        if (expected.colorSpace !== undefined)
            assert_equals(contextAttributes.colorSpace, expected.colorSpace);
        if (expected.desynchronized !== undefined)
            assert_equals(contextAttributes.desynchronized, expected.desynchronized);
        if (expected.willReadFrequently !== undefined)
            assert_equals(contextAttributes.willReadFrequently, expected.willReadFrequently);
        if (expected.colorType !== undefined)
            assert_equals(contextAttributes.colorType, expected.colorType);
        if (expected.toneMapping !== undefined)
            assert_equals(contextAttributes.toneMapping.mode, expected.toneMapping.mode);
    }, testScenario.testDescription);
}

for (var i = 0; i < testScenarios.length; i++)
    runTestScenario(new OffscreenCanvas(100, 100), testScenarios[i]);

done();
