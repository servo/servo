/*

run with
   node split-swizzles.js

notes:

swizzles.test is generated from the C++ based dEQP tests
https://github.com/KhronosGroup/VK-GL-CTS/blob/main/modules/gles3/scripts/gen-swizzles.py

You need to manually add these tests to the 00_test_list.txt

*/

const fs = require('fs');

process.chdir(__dirname);

const testWarning = `\
# WARNING: This file is auto-generated. Do NOT modify it manually, but rather
# modify the generating script file. Otherwise changes will be lost!
# See split-swizzles.js

`;

const swizzlesHTML = fs.readFileSync('swizzles.template', {encoding: 'utf8'});
const swizzles = fs.readFileSync('swizzles.test', {encoding: 'utf8'});
const caseMatches = swizzles.matchAll(/\scase (\w+)_(\w+)_(\w+)\n[\s\S]+?end\n/g);

// quick sanity check
const numCases = swizzles.matchAll('\bcase\b').length;
if (caseMatches.length !== numCases) {
  throw Error(`numCases(${numCases}) does not match caseMatches.length(${caseMatches.length})`);
}

const byType = {}
for (const [str, precision, type, swizzle] of caseMatches) {
  byType[type] = byType[type] || [];
  byType[type].push(str);
}

for (const [type, cases] of Object.entries(byType)) {
  const baseName = `swizzles_${type}`;
  const str = `group ${type}_swizzles "${type} swizzles"\n\n${cases.join('\n\n')}\n\nend # ${type}_swizzles`;
  fs.writeFileSync(`${baseName}.test`, `${testWarning}${str}`);
  fs.writeFileSync(`${baseName}.html`, `<!--\n${testWarning}-->\n${swizzlesHTML.replace("'swizzles'", `'${baseName}'`)}`);
}
