// META: global=window,dedicatedworker

const VIDEO_COLOR_SPACE_SETS = {
  primaries: ['bt709', 'bt470bg', 'smpte170m', 'bt2020', 'smpte432'],
  transfer: ['bt709', 'smpte170m', 'iec61966-2-1', 'linear', 'pq', 'hlg'],
  matrix: ['rgb', 'bt709', 'bt470bg', 'smpte170m', 'bt2020-ncl'],
  fullRange: [true, false],
};

function generateAllCombinations() {
  const keys = Object.keys(VIDEO_COLOR_SPACE_SETS);
  let colorSpaces = [];
  generateAllCombinationsHelper(keys, 0, {}, colorSpaces);
  return colorSpaces;
}

function generateAllCombinationsHelper(keys, keyIndex, colorSpace, results) {
  if (keyIndex >= keys.length) {
    // Push the copied object since the colorSpace will be reused.
    results.push(Object.assign({}, colorSpace));
    return;
  }

  const prop = keys[keyIndex];
  // case 1: Skip this property.
  generateAllCombinationsHelper(keys, keyIndex + 1, colorSpace, results);
  // case 2: Set this property with a valid value.
  for (const val of VIDEO_COLOR_SPACE_SETS[prop]) {
    colorSpace[prop] = val;
    generateAllCombinationsHelper(keys, keyIndex + 1, colorSpace, results);
    delete colorSpace[prop];
  }
}

test(t => {
  let colorSpaces = generateAllCombinations();
  for (const colorSpace of colorSpaces) {
    let vcs = new VideoColorSpace(colorSpace);
    let json = vcs.toJSON();
    for (const k of Object.keys(json)) {
      assert_equals(
        json[k],
        colorSpace.hasOwnProperty(k) ? colorSpace[k] : null
      );
    }
  }
}, 'Test VideoColorSpace toJSON() works.');
