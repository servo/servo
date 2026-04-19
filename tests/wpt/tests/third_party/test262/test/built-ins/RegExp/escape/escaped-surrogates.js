// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-encodeforregexescape
description: Encodes surrogates correctly
info: |
  EncodeForRegExpEscape ( c )

  4. Let toEscape be StringToCodePoints(otherPunctuators).
  5. If toEscape contains c, c is matched by WhiteSpace or LineTerminator, or c has the same numeric value as a leading surrogate or trailing surrogate, then
    a. If c ≤ 0xFF, then
      ...
    b. Let escaped be the empty String.
    c. Let codeUnits be UTF16EncodeCodePoint(c).
    d. For each code unit cu of codeUnits, do
      i. Set escaped to the string-concatenation of escaped and UnicodeEscape(cu).
    e. Return escaped.
  6. Return UTF16EncodeCodePoint(c).
features: [RegExp.escape]
---*/

// Specific surrogate points
assert.sameValue(RegExp.escape('\uD800'), '\\ud800', 'High surrogate \\uD800 is correctly escaped');
assert.sameValue(RegExp.escape('\uDBFF'), '\\udbff', 'High surrogate \\uDBFF is correctly escaped');
assert.sameValue(RegExp.escape('\uDC00'), '\\udc00', 'Low surrogate \\uDC00 is correctly escaped');
assert.sameValue(RegExp.escape('\uDFFF'), '\\udfff', 'Low surrogate \\uDFFF is correctly escaped');

// Leading Surrogates
const highSurrogatesGroup1 = '\uD800\uD801\uD802\uD803\uD804\uD805\uD806\uD807\uD808\uD809\uD80A\uD80B\uD80C\uD80D\uD80E\uD80F';
const highSurrogatesGroup2 = '\uD810\uD811\uD812\uD813\uD814\uD815\uD816\uD817\uD818\uD819\uD81A\uD81B\uD81C\uD81D\uD81E\uD81F';
const highSurrogatesGroup3 = '\uD820\uD821\uD822\uD823\uD824\uD825\uD826\uD827\uD828\uD829\uD82A\uD82B\uD82C\uD82D\uD82E\uD82F';
const highSurrogatesGroup4 = '\uD830\uD831\uD832\uD833\uD834\uD835\uD836\uD837\uD838\uD839\uD83A\uD83B\uD83C\uD83D\uD83E\uD83F';
const highSurrogatesGroup5 = '\uD840\uD841\uD842\uD843\uD844\uD845\uD846\uD847\uD848\uD849\uD84A\uD84B\uD84C\uD84D\uD84E\uD84F';
const highSurrogatesGroup6 = '\uD850\uD851\uD852\uD853\uD854\uD855\uD856\uD857\uD858\uD859\uD85A\uD85B\uD85C\uD85D\uD85E\uD85F';
const highSurrogatesGroup7 = '\uD860\uD861\uD862\uD863\uD864\uD865\uD866\uD867\uD868\uD869\uD86A\uD86B\uD86C\uD86D\uD86E\uD86F';
const highSurrogatesGroup8 = '\uD870\uD871\uD872\uD873\uD874\uD875\uD876\uD877\uD878\uD879\uD87A\uD87B\uD87C\uD87D\uD87E\uD87F';
const highSurrogatesGroup9 = '\uD880\uD881\uD882\uD883\uD884\uD885\uD886\uD887\uD888\uD889\uD88A\uD88B\uD88C\uD88D\uD88E\uD88F';
const highSurrogatesGroup10 = '\uD890\uD891\uD892\uD893\uD894\uD895\uD896\uD897\uD898\uD899\uD89A\uD89B\uD89C\uD89D\uD89E\uD89F';
const highSurrogatesGroup11 = '\uD8A0\uD8A1\uD8A2\uD8A3\uD8A4\uD8A5\uD8A6\uD8A7\uD8A8\uD8A9\uD8AA\uD8AB\uD8AC\uD8AD\uD8AE\uD8AF';
const highSurrogatesGroup12 = '\uD8B0\uD8B1\uD8B2\uD8B3\uD8B4\uD8B5\uD8B6\uD8B7\uD8B8\uD8B9\uD8BA\uD8BB\uD8BC\uD8BD\uD8BE\uD8BF';
const highSurrogatesGroup13 = '\uD8C0\uD8C1\uD8C2\uD8C3\uD8C4\uD8C5\uD8C6\uD8C7\uD8C8\uD8C9\uD8CA\uD8CB\uD8CC\uD8CD\uD8CE\uD8CF';
const highSurrogatesGroup14 = '\uD8D0\uD8D1\uD8D2\uD8D3\uD8D4\uD8D5\uD8D6\uD8D7\uD8D8\uD8D9\uD8DA\uD8DB\uD8DC\uD8DD\uD8DE\uD8DF';
const highSurrogatesGroup15 = '\uD8E0\uD8E1\uD8E2\uD8E3\uD8E4\uD8E5\uD8E6\uD8E7\uD8E8\uD8E9\uD8EA\uD8EB\uD8EC\uD8ED\uD8EE\uD8EF';
const highSurrogatesGroup16 = '\uD8F0\uD8F1\uD8F2\uD8F3\uD8F4\uD8F5\uD8F6\uD8F7\uD8F8\uD8F9\uD8FA\uD8FB\uD8FC\uD8FD\uD8FE\uD8FF';

assert.sameValue(RegExp.escape(highSurrogatesGroup1),  '\\ud800\\ud801\\ud802\\ud803\\ud804\\ud805\\ud806\\ud807\\ud808\\ud809\\ud80a\\ud80b\\ud80c\\ud80d\\ud80e\\ud80f', 'High surrogates group 1 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup2),  '\\ud810\\ud811\\ud812\\ud813\\ud814\\ud815\\ud816\\ud817\\ud818\\ud819\\ud81a\\ud81b\\ud81c\\ud81d\\ud81e\\ud81f', 'High surrogates group 2 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup3),  '\\ud820\\ud821\\ud822\\ud823\\ud824\\ud825\\ud826\\ud827\\ud828\\ud829\\ud82a\\ud82b\\ud82c\\ud82d\\ud82e\\ud82f', 'High surrogates group 3 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup4),  '\\ud830\\ud831\\ud832\\ud833\\ud834\\ud835\\ud836\\ud837\\ud838\\ud839\\ud83a\\ud83b\\ud83c\\ud83d\\ud83e\\ud83f', 'High surrogates group 4 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup5),  '\\ud840\\ud841\\ud842\\ud843\\ud844\\ud845\\ud846\\ud847\\ud848\\ud849\\ud84a\\ud84b\\ud84c\\ud84d\\ud84e\\ud84f', 'High surrogates group 5 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup6),  '\\ud850\\ud851\\ud852\\ud853\\ud854\\ud855\\ud856\\ud857\\ud858\\ud859\\ud85a\\ud85b\\ud85c\\ud85d\\ud85e\\ud85f', 'High surrogates group 6 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup7),  '\\ud860\\ud861\\ud862\\ud863\\ud864\\ud865\\ud866\\ud867\\ud868\\ud869\\ud86a\\ud86b\\ud86c\\ud86d\\ud86e\\ud86f', 'High surrogates group 7 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup8),  '\\ud870\\ud871\\ud872\\ud873\\ud874\\ud875\\ud876\\ud877\\ud878\\ud879\\ud87a\\ud87b\\ud87c\\ud87d\\ud87e\\ud87f', 'High surrogates group 8 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup9),  '\\ud880\\ud881\\ud882\\ud883\\ud884\\ud885\\ud886\\ud887\\ud888\\ud889\\ud88a\\ud88b\\ud88c\\ud88d\\ud88e\\ud88f', 'High surrogates group 9 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup10), '\\ud890\\ud891\\ud892\\ud893\\ud894\\ud895\\ud896\\ud897\\ud898\\ud899\\ud89a\\ud89b\\ud89c\\ud89d\\ud89e\\ud89f', 'High surrogates group 10 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup11), '\\ud8a0\\ud8a1\\ud8a2\\ud8a3\\ud8a4\\ud8a5\\ud8a6\\ud8a7\\ud8a8\\ud8a9\\ud8aa\\ud8ab\\ud8ac\\ud8ad\\ud8ae\\ud8af', 'High surrogates group 11 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup12), '\\ud8b0\\ud8b1\\ud8b2\\ud8b3\\ud8b4\\ud8b5\\ud8b6\\ud8b7\\ud8b8\\ud8b9\\ud8ba\\ud8bb\\ud8bc\\ud8bd\\ud8be\\ud8bf', 'High surrogates group 12 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup13), '\\ud8c0\\ud8c1\\ud8c2\\ud8c3\\ud8c4\\ud8c5\\ud8c6\\ud8c7\\ud8c8\\ud8c9\\ud8ca\\ud8cb\\ud8cc\\ud8cd\\ud8ce\\ud8cf', 'High surrogates group 13 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup14), '\\ud8d0\\ud8d1\\ud8d2\\ud8d3\\ud8d4\\ud8d5\\ud8d6\\ud8d7\\ud8d8\\ud8d9\\ud8da\\ud8db\\ud8dc\\ud8dd\\ud8de\\ud8df', 'High surrogates group 14 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup15), '\\ud8e0\\ud8e1\\ud8e2\\ud8e3\\ud8e4\\ud8e5\\ud8e6\\ud8e7\\ud8e8\\ud8e9\\ud8ea\\ud8eb\\ud8ec\\ud8ed\\ud8ee\\ud8ef', 'High surrogates group 15 are correctly escaped');
assert.sameValue(RegExp.escape(highSurrogatesGroup16), '\\ud8f0\\ud8f1\\ud8f2\\ud8f3\\ud8f4\\ud8f5\\ud8f6\\ud8f7\\ud8f8\\ud8f9\\ud8fa\\ud8fb\\ud8fc\\ud8fd\\ud8fe\\ud8ff', 'High surrogates group 16 are correctly escaped');

// Trailing Surrogates
const lowSurrogatesGroup1 = '\uDC00\uDC01\uDC02\uDC03\uDC04\uDC05\uDC06\uDC07\uDC08\uDC09\uDC0A\uDC0B\uDC0C\uDC0D\uDC0E\uDC0F';
const lowSurrogatesGroup2 = '\uDC10\uDC11\uDC12\uDC13\uDC14\uDC15\uDC16\uDC17\uDC18\uDC19\uDC1A\uDC1B\uDC1C\uDC1D\uDC1E\uDC1F';
const lowSurrogatesGroup3 = '\uDC20\uDC21\uDC22\uDC23\uDC24\uDC25\uDC26\uDC27\uDC28\uDC29\uDC2A\uDC2B\uDC2C\uDC2D\uDC2E\uDC2F';
const lowSurrogatesGroup4 = '\uDC30\uDC31\uDC32\uDC33\uDC34\uDC35\uDC36\uDC37\uDC38\uDC39\uDC3A\uDC3B\uDC3C\uDC3D\uDC3E\uDC3F';
const lowSurrogatesGroup5 = '\uDC40\uDC41\uDC42\uDC43\uDC44\uDC45\uDC46\uDC47\uDC48\uDC49\uDC4A\uDC4B\uDC4C\uDC4D\uDC4E\uDC4F';
const lowSurrogatesGroup6 = '\uDC50\uDC51\uDC52\uDC53\uDC54\uDC55\uDC56\uDC57\uDC58\uDC59\uDC5A\uDC5B\uDC5C\uDC5D\uDC5E\uDC5F';
const lowSurrogatesGroup7 = '\uDC60\uDC61\uDC62\uDC63\uDC64\uDC65\uDC66\uDC67\uDC68\uDC69\uDC6A\uDC6B\uDC6C\uDC6D\uDC6E\uDC6F';
const lowSurrogatesGroup8 = '\uDC70\uDC71\uDC72\uDC73\uDC74\uDC75\uDC76\uDC77\uDC78\uDC79\uDC7A\uDC7B\uDC7C\uDC7D\uDC7E\uDC7F';
const lowSurrogatesGroup9 = '\uDC80\uDC81\uDC82\uDC83\uDC84\uDC85\uDC86\uDC87\uDC88\uDC89\uDC8A\uDC8B\uDC8C\uDC8D\uDC8E\uDC8F';
const lowSurrogatesGroup10 = '\uDC90\uDC91\uDC92\uDC93\uDC94\uDC95\uDC96\uDC97\uDC98\uDC99\uDC9A\uDC9B\uDC9C\uDC9D\uDC9E\uDC9F';
const lowSurrogatesGroup11 = '\uDCA0\uDCA1\uDCA2\uDCA3\uDCA4\uDCA5\uDCA6\uDCA7\uDCA8\uDCA9\uDCAA\uDCAB\uDCAC\uDCAD\uDCAE\uDCAF';
const lowSurrogatesGroup12 = '\uDCB0\uDCB1\uDCB2\uDCB3\uDCB4\uDCB5\uDCB6\uDCB7\uDCB8\uDCB9\uDCBA\uDCBB\uDCBC\uDCBD\uDCBE\uDCBF';
const lowSurrogatesGroup13 = '\uDCC0\uDCC1\uDCC2\uDCC3\uDCC4\uDCC5\uDCC6\uDCC7\uDCC8\uDCC9\uDCCA\uDCCB\uDCCC\uDCCD\uDCCE\uDCCF';
const lowSurrogatesGroup14 = '\uDCD0\uDCD1\uDCD2\uDCD3\uDCD4\uDCD5\uDCD6\uDCD7\uDCD8\uDCD9\uDCDA\uDCDB\uDCDC\uDCDD\uDCDE\uDCDF';
const lowSurrogatesGroup15 = '\uDCE0\uDCE1\uDCE2\uDCE3\uDCE4\uDCE5\uDCE6\uDCE7\uDCE8\uDCE9\uDCEA\uDCEB\uDCEC\uDCED\uDCEE\uDCEF';
const lowSurrogatesGroup16 = '\uDCF0\uDCF1\uDCF2\uDCF3\uDCF4\uDCF5\uDCF6\uDCF7\uDCF8\uDCF9\uDCFA\uDCFB\uDCFC\uDCFD\uDCFE\uDCFF';

assert.sameValue(RegExp.escape(lowSurrogatesGroup1),  '\\udc00\\udc01\\udc02\\udc03\\udc04\\udc05\\udc06\\udc07\\udc08\\udc09\\udc0a\\udc0b\\udc0c\\udc0d\\udc0e\\udc0f', 'Low surrogates group 1 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup2),  '\\udc10\\udc11\\udc12\\udc13\\udc14\\udc15\\udc16\\udc17\\udc18\\udc19\\udc1a\\udc1b\\udc1c\\udc1d\\udc1e\\udc1f', 'Low surrogates group 2 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup3),  '\\udc20\\udc21\\udc22\\udc23\\udc24\\udc25\\udc26\\udc27\\udc28\\udc29\\udc2a\\udc2b\\udc2c\\udc2d\\udc2e\\udc2f', 'Low surrogates group 3 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup4),  '\\udc30\\udc31\\udc32\\udc33\\udc34\\udc35\\udc36\\udc37\\udc38\\udc39\\udc3a\\udc3b\\udc3c\\udc3d\\udc3e\\udc3f', 'Low surrogates group 4 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup5),  '\\udc40\\udc41\\udc42\\udc43\\udc44\\udc45\\udc46\\udc47\\udc48\\udc49\\udc4a\\udc4b\\udc4c\\udc4d\\udc4e\\udc4f', 'Low surrogates group 5 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup6),  '\\udc50\\udc51\\udc52\\udc53\\udc54\\udc55\\udc56\\udc57\\udc58\\udc59\\udc5a\\udc5b\\udc5c\\udc5d\\udc5e\\udc5f', 'Low surrogates group 6 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup7),  '\\udc60\\udc61\\udc62\\udc63\\udc64\\udc65\\udc66\\udc67\\udc68\\udc69\\udc6a\\udc6b\\udc6c\\udc6d\\udc6e\\udc6f', 'Low surrogates group 7 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup8),  '\\udc70\\udc71\\udc72\\udc73\\udc74\\udc75\\udc76\\udc77\\udc78\\udc79\\udc7a\\udc7b\\udc7c\\udc7d\\udc7e\\udc7f', 'Low surrogates group 8 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup9),  '\\udc80\\udc81\\udc82\\udc83\\udc84\\udc85\\udc86\\udc87\\udc88\\udc89\\udc8a\\udc8b\\udc8c\\udc8d\\udc8e\\udc8f', 'Low surrogates group 9 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup10), '\\udc90\\udc91\\udc92\\udc93\\udc94\\udc95\\udc96\\udc97\\udc98\\udc99\\udc9a\\udc9b\\udc9c\\udc9d\\udc9e\\udc9f', 'Low surrogates group 10 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup11), '\\udca0\\udca1\\udca2\\udca3\\udca4\\udca5\\udca6\\udca7\\udca8\\udca9\\udcaa\\udcab\\udcac\\udcad\\udcae\\udcaf', 'Low surrogates group 11 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup12), '\\udcb0\\udcb1\\udcb2\\udcb3\\udcb4\\udcb5\\udcb6\\udcb7\\udcb8\\udcb9\\udcba\\udcbb\\udcbc\\udcbd\\udcbe\\udcbf', 'Low surrogates group 12 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup13), '\\udcc0\\udcc1\\udcc2\\udcc3\\udcc4\\udcc5\\udcc6\\udcc7\\udcc8\\udcc9\\udcca\\udccb\\udccc\\udccd\\udcce\\udccf', 'Low surrogates group 13 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup14), '\\udcd0\\udcd1\\udcd2\\udcd3\\udcd4\\udcd5\\udcd6\\udcd7\\udcd8\\udcd9\\udcda\\udcdb\\udcdc\\udcdd\\udcde\\udcdf', 'Low surrogates group 14 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup15), '\\udce0\\udce1\\udce2\\udce3\\udce4\\udce5\\udce6\\udce7\\udce8\\udce9\\udcea\\udceb\\udcec\\udced\\udcee\\udcef', 'Low surrogates group 15 are correctly escaped');
assert.sameValue(RegExp.escape(lowSurrogatesGroup16), '\\udcf0\\udcf1\\udcf2\\udcf3\\udcf4\\udcf5\\udcf6\\udcf7\\udcf8\\udcf9\\udcfa\\udcfb\\udcfc\\udcfd\\udcfe\\udcff', 'Low surrogates group 16 are correctly escaped');
