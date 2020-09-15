'use strict';

const TEST_SIZE_CATEGORY = {
  // Fonts with file smaller than 1MiB.
  small: 'small',
  // Fonts with file between 1 and 20MiB.
  medium: 'medium',
  // Fonts with file larger than 20MiB.
  large: 'large',
};

const MAC_FONTS = [
  {
    postscriptName: 'Monaco',
    fullName: 'Monaco',
    family: 'Monaco',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      // Tables related to TrueType.
      'cvt ', 'glyf', 'loca', 'prep', 'gasp',
    ],
  },
  {
    postscriptName: 'Menlo-Regular',
    fullName: 'Menlo Regular',
    family: 'Menlo',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  {
    postscriptName: 'Menlo-Bold',
    fullName: 'Menlo Bold',
    family: 'Menlo',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  {
    postscriptName: 'Menlo-BoldItalic',
    fullName: 'Menlo Bold Italic',
    family: 'Menlo',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  // Indic.
  {
    postscriptName: 'GujaratiMT',
    fullName: 'Gujarati MT',
    family: 'Gujarati MT',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  {
    postscriptName: 'GujaratiMT-Bold',
    fullName: 'Gujarati MT Bold',
    family: 'Gujarati MT',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  {
    postscriptName: 'DevanagariMT',
    fullName: 'Devanagari MT',
    family: 'Devanagari MT',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  {
    postscriptName: 'DevanagariMT-Bold',
    fullName: 'Devanagari MT Bold',
    family: 'Devanagari MT',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  // Japanese.
  {
    postscriptName: 'HiraMinProN-W3',
    fullName: 'Hiragino Mincho ProN W3',
    family: 'Hiragino Mincho ProN',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'CFF ', 'VORG',
    ],
  },
  {
    postscriptName: 'HiraMinProN-W6',
    fullName: 'Hiragino Mincho ProN W6',
    family: 'Hiragino Mincho ProN',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'CFF ', 'VORG',
    ],
  },
  // Korean.
  {
    postscriptName: 'AppleGothic',
    fullName: 'AppleGothic Regular',
    family: 'AppleGothic',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'cvt ', 'glyf', 'loca',
    ],
  },
  {
    postscriptName: 'AppleMyungjo',
    fullName: 'AppleMyungjo Regular',
    family: 'AppleMyungjo',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      'cvt ', 'glyf', 'loca',
    ],
  },
  // Chinese.
  {
    postscriptName: 'STHeitiTC-Light',
    fullName: 'Heiti TC Light',
    family: 'Heiti TC',
    label: TEST_SIZE_CATEGORY.large,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  {
   postscriptName: 'STHeitiTC-Medium',
    fullName: 'Heiti TC Medium',
    family: 'Heiti TC',
    label: TEST_SIZE_CATEGORY.large,
    expectedTables: [
      'cvt ', 'glyf', 'loca', 'prep',
    ],
  },
  // Bitmap.
  {
    postscriptName: 'AppleColorEmoji',
    fullName: 'Apple Color Emoji',
    family: 'Apple Color Emoji',
    label: TEST_SIZE_CATEGORY.large,
    expectedTables: [
      'glyf', 'loca',
      // Tables related to Bitmap Glyphs.
      'sbix',
    ],
  },
];

const WIN_FONTS = [
  {
    postscriptName: 'Verdana',
    fullName: 'Verdana',
    family: 'Verdana',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      // Tables related to TrueType.
      'cvt ',
      'glyf',
      'loca',
      'prep',
      'gasp',
    ],
  },
  {
    postscriptName: 'Verdana-Bold',
    fullName: 'Verdana Bold',
    family: 'Verdana',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      // Tables related to TrueType.
      'cvt ',
      'glyf',
      'loca',
      'prep',
      'gasp',
    ],
  },
  {
    postscriptName: 'Verdana-Italic',
    fullName: 'Verdana Italic',
    family: 'Verdana',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      // Tables related to TrueType.
      'cvt ',
      'glyf',
      'loca',
      'prep',
      'gasp',
    ],
  },
  // Korean.
  {
    postscriptName: 'MalgunGothicBold',
    fullName: 'Malgun Gothic Bold',
    family: 'Malgun Gothic',
    label: TEST_SIZE_CATEGORY.medium,
    expectedTables: [
      // Tables related to TrueType.
      'cvt ',
      'glyf',
      'loca',
      'prep',
      'gasp',
    ],
  },
  // Chinese.
  {
    postscriptName: 'MicrosoftYaHei',
    fullName: 'Microsoft YaHei',
    family: 'Microsoft YaHei',
    label: TEST_SIZE_CATEGORY.medium,
  },
  {
    postscriptName: 'MicrosoftYaHei-Bold',
    fullName: 'Microsoft YaHei Bold',
    family: 'Microsoft YaHei',
    label: TEST_SIZE_CATEGORY.medium,
  },
];

const LINUX_FONTS = [
  {
    postscriptName: 'Ahem',
    fullName: 'Ahem',
    family: 'Ahem',
    label: TEST_SIZE_CATEGORY.small,
    expectedTables: [
      // Tables related to TrueType.
      'cvt ',
      'glyf',
      'loca',
      'prep',
      'gasp',
    ],
  },
];

// The OpenType spec mentions that the follow tables are required for a font to
// function correctly. We'll have all the tables listed except for OS/2, which
// is not present in all fonts on Mac OS.
// https://docs.microsoft.com/en-us/typography/opentype/spec/otff#font-tables
const BASE_TABLES = [
  'cmap',
  'head',
  'hhea',
  'hmtx',
  'maxp',
  'name',
  'post',
];

function getEnumerationTestSet(options) {
  options = Object.assign({
    labelFilter: [],
  }, options);

  // Verify (by font family) that some standard fonts have been returned.
  let platform;
  if (navigator.platform.indexOf("Win") !== -1) {
    platform = 'win';
  } else if (navigator.platform.indexOf("Mac") !== -1) {
    platform = 'mac';
  } else if (navigator.platform.indexOf("Linux") !== -1) {
    platform = 'linux';
  } else {
    platform = 'generic';
  }

  assert_not_equals(platform, 'generic', 'Platform must be detected.');

  let output = [];
  if (platform === 'mac') {
    output = MAC_FONTS;
  } else if (platform === 'win') {
    output = WIN_FONTS;
  } else if (platform === 'linux') {
    // Also includes ChromeOS, on which navigator.platform starts with 'Linux'.
    output = LINUX_FONTS;
  }

  if (options.labelFilter.length && output.length) {
    const labelFilter = new Set(options.labelFilter);
    output = output.filter(f => labelFilter.has(f.label));
  }

  return output;
}

function getMoreExpectedTables(expectations) {
  const output = {};
  for (const f of expectations) {
    if (f.expectedTables) {
      output[f.postscriptName] = f.expectedTables;
    }
  }
  return output;
}

async function filterEnumeration(iterator, expectedFonts) {
  const nameSet = new Set();
  for (const e of expectedFonts) {
    nameSet.add(e.postscriptName);
  }

  const output = [];
  for await (const f of iterator) {
    if (nameSet.has(f.postscriptName)) {
      output.push(f);
    }
  }

  const numGot = output.length;
  const numExpected = Object.keys(expectedFonts).length;
  assert_equals(numGot, numExpected, `Got ${numGot} fonts, expected ${numExpected}.`);

  return output;
}

function assert_fonts_exist(availableFonts, expectedFonts) {
  const postscriptNameSet = new Set();
  const fullNameSet = new Set();
  const familySet = new Set();

  for (const f of expectedFonts) {
    postscriptNameSet.add(f.postscriptName);
    fullNameSet.add(f.fullName);
    familySet.add(f.family);
  }

  for (const f of availableFonts) {
    postscriptNameSet.delete(f.postscriptName);
    fullNameSet.delete(f.fullName);
    familySet.delete(f.family);
  }

  assert_equals(postscriptNameSet.size, 0,
              `Missing Postscript names: ${setToString(postscriptNameSet)}.`);
  assert_equals(fullNameSet.size, 0,
              `Missing Full names: ${setToString(fullNameSet)}.`);
  assert_equals(familySet.size, 0,
              `Missing Families: ${setToString(familySet)}.`);
}

function assert_font_has_tables(name, tables, expectedTables) {
  for (const t of expectedTables) {
    assert_equals(t.length, 4,
                "Table names are always 4 characters long.");
    assert_true(tables.has(t),
                `Font ${name} did not have required table ${t}.`);
    assert_greater_than(tables.get(t).size, 0,
                `Font ${name} has table ${t} of size 0.`);
  }
}

function setToString(set) {
  const items = Array.from(set);
  return JSON.stringify(items);
}

async function parseFontData(fontBlob) {
  const fontInfo = {
    errors: [],
    numTables: 0,
  };
  const versionTag = await getTag(fontBlob, 0);

  fontInfo.version = sfntVersionInfo(versionTag);
  if (fontInfo.version === 'UNKNOWN') {
    fontInfo.errors.push(`versionTag: "${versionTag}"`);
  }

  const numTables = await getUint16(fontBlob, 4);
  [fontInfo.tables, fontInfo.tableMeta] = await getTableData(fontBlob, numTables);

  return fontInfo;
}

async function getTableData(fontBlob, numTables) {
  const dataMap = new Map();
  const metaMap = new Map();
  let blobOffset = 12;

  for (let i = 0; i < numTables; i++) {
    const tag = await getTag(fontBlob, blobOffset);
    const checksum = await getUint32(fontBlob, blobOffset + 4);
    const offset = await getUint32(fontBlob, blobOffset + 8);
    const size = await getUint32(fontBlob, blobOffset + 12);
    const tableBlob = fontBlob.slice(offset, offset + size);
    dataMap.set(tag, tableBlob);
    metaMap.set(tag, {checksum, offset, size});
    blobOffset += 16;
  }

  return [dataMap, metaMap];
}

function sfntVersionInfo(version) {
  // Spec: https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
  switch (version) {
  case '\x00\x01\x00\x00':
  case 'true':
  case 'typ1':
    return 'truetype';
  case 'OTTO':
    return 'cff';
  default:
    return 'UNKNOWN';
  }
}

async function getTag(blob, offset) {
  return (new TextDecoder).decode(
    await blob.slice(offset, offset + 4).arrayBuffer());
}

async function getUint16(blob, offset) {
  const slice = blob.slice(offset, offset + 2);
  const buf = await slice.arrayBuffer();
  const dataView = new DataView(buf);
  return dataView.getUint16(0);
}

async function getUint32(blob, offset) {
  const slice = blob.slice(offset, offset + 4);
  const buf = await slice.arrayBuffer();
  const dataView = new DataView(buf);
  return dataView.getUint32(0);
}

function promiseDocumentReady() {
  return new Promise(resolve => {
    if (document.readyState === 'complete') {
      resolve();
    }
    window.addEventListener('load', () => {
      resolve();
    }, {once: true});
  });
}

function isPlatformSupported() {
  if (navigator.platform.indexOf('Mac') != -1 ||
      navigator.platform.indexOf('Win') != -1 ||
      navigator.platform.indexOf('Linux') != -1) {
    return true;
  }
  return false;
}

async function simulateUserActivation() {
  await promiseDocumentReady();
  return new Promise(resolve => {
    const button = document.createElement('button');
    button.textContent = 'Click to enumerate fonts';
    button.style.fontSize = '40px';
    button.onclick = () => {
      document.body.removeChild(button);
      resolve();
    };
    document.body.appendChild(button);
    test_driver.click(button);
  });
}

function font_access_test(test_function, name, properties) {
  return promise_test(async (t) => {
    await test_driver.set_permission({name: 'font-access'}, 'granted');
    await simulateUserActivation();
    await test_function(t, name, properties);
  });
}
