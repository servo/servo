/*

run with
   node split-conversions.js

notes:

conversions.test is generated from the C++ based dEQP tests
https://github.com/KhronosGroup/VK-GL-CTS/blob/main/modules/gles3/scripts/gen-conversions.py

You need to manually add these tests to the 00_test_list.txt

*/

const fs = require('fs');

process.chdir(__dirname);

const testWarning = `\
# WARNING: This file is auto-generated. Do NOT modify it manually, but rather
# modify the generating script file. Otherwise changes will be lost!
# See split-conversions.js

`;

const conversionsHTML = fs.readFileSync('conversions.template', {encoding: 'utf8'});
const conversions = fs.readFileSync('conversions.test', {encoding: 'utf8'});
const groupMatches = conversions.matchAll(/group ([a-zA-Z0-9_]+) [\s\S]+?end #.*?\n/g);
for (const [str, groupName] of groupMatches) {
  const baseName = `conversions_${groupName}`;
  fs.writeFileSync(`${baseName}.test`, `${testWarning}${str}`);
  fs.writeFileSync(`${baseName}.html`, `<!--\n${testWarning}-->\n${conversionsHTML.replace("'conversions'", `'${baseName}'`)}`);
}
