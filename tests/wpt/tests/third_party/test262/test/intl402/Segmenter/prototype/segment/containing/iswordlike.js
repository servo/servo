// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the isWordLike in the result when granularity is not "word".
info: |
    %Segments.prototype%.containing ( index )

    10. Return ! CreateSegmentDataObject(segmenter, string, startIndex, endIndex).

    CreateSegmentDataObject ( segmenter, string, startIndex, endIndex )
    11. If granularity is "word", then
     a. Let isWordLike be a Boolean value indicating whether the segment in string is "word-like" according to locale segmenter.[[Locale]].
     b. Perform ! CreateDataPropertyOrThrow(result, "isWordLike", isWordLike).

includes: [compareArray.js]
features: [Intl.Segmenter]
---*/

const other_granularities = [undefined, "grapheme", "sentence"];
// Some text
const inputs = [
    "Hello world!", // English
    "Jedovatou mambu objevila žena v zahrádkářské kolonii.", // Czech
    "Việt Nam: Nhất thể hóa sẽ khác Trung Quốc?",  // Vietnamese
    "Σοβαρές ενστάσεις Κομισιόν για τον προϋπολογισμό της Ιταλίας", // Greek
    "Решение Индии о покупке российских С-400 расценили как вызов США",  // Russian
    "הרופא שהציל נשים והנערה ששועבדה ע",  // Hebrew,
    "ترامب للملك سلمان: أنا جاد للغاية.. عليك دفع المزيد", // Arabic
    "भारत की एस 400 मिसाइल के मुकाबले पाक की थाड, जानें कौन कितना ताकतवर",  //  Hindi
    "ரெட் அலர்ட் எச்சரிக்கை; புதுச்சேரியில் நாளை அரசு விடுமுறை!", // Tamil
    "'ఉత్తర్వులు అందే వరకు ఓటర్ల తుది జాబితాను వెబ్‌సైట్లో పెట్టవద్దు'", // Telugu
    "台北》抹黑柯P失敗？朱學恒酸：姚文智氣pupu嗆大老闆", // Chinese
    "วัดไทรตีระฆังเบาลงช่วงเข้าพรรษา เจ้าอาวาสเผยคนร้องเรียนรับผลกรรมแล้ว",  // Thai
    "九州北部の一部が暴風域に入りました(日直予報士 2018年10月06日) - 日本気象協会 tenki.jp",  // Japanese
    "법원 “다스 지분 처분권·수익권 모두 MB가 보유”", // Korean
];

other_granularities.forEach(
    function(granularity) {
      const segmenter = new Intl.Segmenter(undefined, {granularity});
      inputs.forEach(function(input) {
        const segment = segmenter.segment(input);
        for (let index = 0; index < input.length; index++) {
          const result = segment.containing(index);
          const msg =
              `granularity: ${granularity} input: ${input} containing(${index})`;
          assert.sameValue(true, result.index >= 0, `${msg} index >= 0`);
          assert.sameValue(true, result.index < input.length, `${msg} index`);
          assert.sameValue("string", typeof result.input, `${msg} input`);
          assert.sameValue(undefined, result.isWordLike,
              `${msg} isWordLike should be undefined`);
          assert.compareArray(Object.getOwnPropertyNames(result), ["segment", "index", "input"]);
        }
      });
    });
