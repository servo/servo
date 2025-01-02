/* Script used by descriptor-ranges.html and descriptor-ranges-ref.html
   to create @font-face rules and test elements for a collection of
   font-matching testcases. */

// To create unique font-family names for each testcase.
let serial = 0;

// Accumulators for the lists of @font-face rules and test elements.
let fontFaceRules = "";
let testElements = "";

// Create a <div> element with the font properties to match. Its text reports
// the property-value and corresponding pair of descriptors being tested.
// (The associated createFontFaceRules() function is defined separately in the
// test and reference files.)
function createTestElement(family, weight, style, stretch, value, expected, unexpected) {
    return `<div style="font-family:${family}; font-weight:${weight}; font-style:${style}; font-stretch:${stretch};">` +
           `${family} ${value} (${expected} vs ${unexpected})</div>\n`;
}

// Create testcases for the given descriptor.
// Each testcase has a test property value, and a list of @font-face descriptors
// to be matched against the property, where each descriptor in the list should
// be preferred over the next.
function testDescriptor(descriptorName, testCases) {
    testElements += `<div style="background:yellow;padding:0.5em">Tests of ${descriptorName} descriptor:</div>\n`;
    testCases.forEach(function (testCase) {
        // Go though test cases, checking each descriptor has higher priority than next in the list
        for (let i = 0; i < testCase.testDescriptors.length - 1; i++) {
            serial++;
            let expectedMatch   = testCase.testDescriptors[i];
            let unexpectedMatch = testCase.testDescriptors[i + 1];
            let familyName = "test_" + serial;
            fontFaceRules += createFontFaceRules(familyName, descriptorName, expectedMatch, unexpectedMatch);
            let testWeight  = (descriptorName == "font-weight")  ? testCase.value : "normal";
            let testStyle   = (descriptorName == "font-style")   ? testCase.value : "normal";
            let testStretch = (descriptorName == "font-stretch") ? testCase.value : "normal";
            testElements += createTestElement(familyName, testWeight, testStyle, testStretch,
                                              testCase.value, expectedMatch, unexpectedMatch);
        }
    });
}

// Testcases (from web-platform/tests/css/css-fonts/variations/at-font-face-font-matching.html,
// with a couple of extras). In each case, for the given property value, the testDescriptors
// are listed from 'best' to 'worse' match, as evaluated by the font-matching algorithm in
// https://drafts.csswg.org/css-fonts-4/#font-style-matching.
testDescriptor("font-weight", [
    { value: "400", testDescriptors: ["400", "450 460", "500", "350 399", "351 398", "501 550", "502 560"] },
    { value: "430", testDescriptors: ["420 440", "450 460", "500", "400 425", "350 399", "340 398", "501 550", "502 560"] },
    { value: "500", testDescriptors: ["500", "450 460", "400", "350 399", "351 398", "501 550", "502 560"] },
    { value: "501", testDescriptors: ["501", "502 510", "503 520", "500", "450 460", "390 410", "300 350"] },
    { value: "399", testDescriptors: ["350 399", "340 360", "200 300", "400", "450 460", "500 501", "502 510"] },
    { value: "350", testDescriptors: ["200 300", "250 280", "420 450", "430 440", "445"] },
    { value: "550", testDescriptors: ["600 800", "700 900", "420 450", "430 440", "425"] }
]);

testDescriptor("font-stretch", [
    { value: "100%", testDescriptors: ["100%", "110% 120%", "115% 116%"] },
    { value: "110%", testDescriptors: ["110% 120%", "115% 116%", "105%", "100%", "50% 80%", "60% 70%"] },
    { value: "90%",  testDescriptors: ["90% 100%", "50% 80%", "60% 70%", "110% 140%", "120% 130%"] },
]);

testDescriptor("font-style", [
    { value: "normal",         testDescriptors: ["normal", "oblique 0deg", "oblique 10deg 40deg", "oblique 20deg 30deg", "oblique -50deg -20deg", "oblique -40deg -30deg" ] },
    { value: "italic",         testDescriptors: ["italic", "oblique 20deg", "oblique 30deg 60deg", "oblique 40deg 50deg", "oblique 5deg 10deg", "oblique 5deg", "normal", "oblique 0deg", "oblique -60deg -30deg", "oblique -50deg -40deg" ] },
    { value: "oblique 20deg",  testDescriptors: ["oblique 20deg", "oblique 30deg 60deg", "oblique 40deg 50deg", "oblique 10deg", "italic", "oblique 0deg", "oblique -50deg -20deg", "oblique -40deg -30deg" ] },
    { value: "oblique 21deg",  testDescriptors: ["oblique 21deg", "oblique 30deg 60deg", "oblique 40deg 50deg", "oblique 20deg", "oblique 10deg", "italic", "oblique 0deg",  "oblique -50deg -20deg", "oblique -40deg -30deg" ] },
    { value: "oblique 10deg",  testDescriptors: ["oblique 10deg", "oblique 5deg", "oblique 15deg 20deg", "oblique 30deg 60deg", "oblique 40deg 50deg", "italic", "oblique 0deg", "oblique -50deg -20deg", "oblique -40deg -30deg" ] },
    { value: "oblique 0deg",   testDescriptors: ["oblique 0deg", "oblique 5deg", "oblique 15deg 20deg", "oblique 30deg 60deg", "oblique 40deg 50deg", "italic", "oblique -50deg -20deg", "oblique -40deg -30deg" ] },
    { value: "oblique -10deg", testDescriptors: ["oblique -10deg", "oblique -5deg", "oblique -1deg 0deg", "oblique -20deg -15deg", "oblique -60deg -30deg", "oblique -50deg -40deg", "italic", "oblique 0deg 10deg", "oblique 40deg 50deg" ] },
    { value: "oblique -20deg", testDescriptors: ["oblique -20deg", "oblique -60deg -40deg", "oblique -10deg", "italic", "oblique 0deg", "oblique 30deg 60deg", "oblique 40deg 50deg"] },
    { value: "oblique -21deg", testDescriptors: ["oblique -21deg", "oblique -60deg -40deg", "oblique -10deg", "italic", "oblique 0deg", "oblique 30deg 60deg", "oblique 40deg 50deg"] },
]);

// Stuff the @font-face rules and test elements into the document.
// Any testcases that render Ahem glyphs are failures.
document.getElementById("dynamicStyles").innerHTML = fontFaceRules;
document.getElementById("testContents").innerHTML = testElements;
