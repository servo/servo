// This is a node.js program to generate text-orientation-script test files.
'use strict';

(function (exports) {
  var ejs = require("ejs");
  var fs = require("fs");
  var unicodeData = require("./unicode-data.js");

  class Generator {
    constructor(rangesByVO, gc, blocks) {
      this.rangesByVO = rangesByVO;
      this.gc = gc;
      this.blocks = blocks;
      this.charactersPerLine = 32;
    }
    generate(argv) {
      var codePointsByVO = {};
      var gc = this.gc;
      var skipFunc = this.createSkipFunc(argv.noskip);
      for (var value in this.rangesByVO)
        codePointsByVO[value] = unicodeData.codePointsFromRanges(this.rangesByVO[value], skipFunc);

      this.codePointsByVO = codePointsByVO;
      var template = fs.readFileSync("text-orientation-script.ejs", "utf-8");
      this.template = ejs.compile(template);

      if (!argv.nocombo)
        this.generateFile();
      if (argv.nochild)
        return;

      var pageSize = this.charactersPerLine * 64;
      var fileIndex = 0;
      for (var vo in codePointsByVO) {
        var codePoints = codePointsByVO[vo];
        var limit = codePoints.length;
        var pages = Math.ceil(limit / pageSize);
        for (var min = 0, page = 1; min < limit; ++page, ++fileIndex) {
          var nextLimit = Math.min(limit, min + pageSize);
          this.codePointsByVO = {};
          this.codePointsByVO[vo] = codePoints.slice(min, nextLimit);
          this.generateFile(vo, fileIndex, page, pages);
          min = nextLimit;
        }
      }
    }
    generateFile(vo, fileIndex, page, pages) {
      var path = "../../text-orientation-script-001";
      this.title = "Test orientation of characters";
      this.flags = "dom";
      // if (fileIndex)
      //     path += "-" + padZero(fileIndex, 3);
      if (fileIndex === undefined)
        this.flags += " combo";
      else
        path += affixFromIndex(fileIndex);
      if (vo) {
        this.title += " where vo=" + vo;
        var codePoints = this.codePointsByVO[vo];
        var rangeText = codePoints.length + " code points in U+" +
          unicodeData.toHex(codePoints[0]) + "-" +
          unicodeData.toHex(codePoints[codePoints.length - 1]);
        if (page && pages > 1)
          rangeText = "#" + page + "/" + pages + ", " + rangeText;
        this.title += " (" + rangeText + ")";
      }
      path += ".html";
      console.log("Writing " + path + ": " + this.title);
      var output = fs.openSync(path, "w");
      fs.writeSync(output, this.template(this));
      fs.closeSync(output);
    }
    generateRefTest() {
      var template = fs.readFileSync("text-orientation-ref.ejs", "utf-8");
      this.template = ejs.compile(template);
      this.codePointRanges = [
        [0x0020, 0x007E],
        [0x3000, 0x30FF],
        [0x4E00, 0x4E1F],
        [0xFF01, 0xFF60],
      ];
      var writingModes = [
        { key: "vlr", value: "vertical-lr" },
        { key: "vrl", value: "vertical-rl" },
      ];
      var voByCodePoint = unicodeData.arrayFromRangesByValue(this.rangesByVO);
      var R = 0x0041, U = 0x56FD;
      var textOrientations = [
        { value: "mixed", ref: function (ch) { return voByCodePoint[ch] == "R" ? R : U; } },
        { value: "sideways", ref: function (ch) { return R; } },
        { value: "upright", ref: function (ch) { return U; } },
      ];
      var self = this;
      writingModes.forEach(function (writingMode) {
        self.writingMode = writingMode.value;
        textOrientations.forEach(function (textOrientation) {
          self.textOrientation = textOrientation.value;
          self.title = "writing-mode: " + self.writingMode + "; text-orientation: " + self.textOrientation;
          var key = textOrientation.value + "-" + writingMode.key;
          self.generateRefTestFile(key, false);
          self.generateRefTestFile(key, true, textOrientation.ref);
        });
      });
    }
    generateRefTestFile(key, isReference, mapCodePointForRendering) {
      var name = "text-orientation-" + key + "-100";
      var path = name + ".html";
      var reference = name + "-ref.html";
      if (isReference) {
        path = "../../" + reference;
        this.reference = null;
      } else {
        path = "../../" + path;
        this.reference = reference;
      }
      console.log("Writing " + path + ": " + this.title);
      var skipFunc0 = this.createSkipFunc(true);
      // Workaround CSS test harness bug that double-escape &lt; and &gt;.
      var skipFunc = c => c == 0x3C || c == 0x3E || skipFunc0(c);
      this.codePointsFromRangeForRendering = mapCodePointForRendering
        ? range => unicodeData.codePointsFromRanges(range, skipFunc).map(mapCodePointForRendering)
        : range => unicodeData.codePointsFromRanges(range, skipFunc);
      var output = fs.openSync(path, "w");
      fs.writeSync(output, this.template(this));
      fs.closeSync(output);
    }
    headingFromRange(range) {
      return "U+" + unicodeData.toHex(range[0]) + "-" + unicodeData.toHex(range[range.length - 1]);
    }
    createSkipFunc(noSkip) {
      var gc = this.gc;
      function skipCombiningMarks(code) {
        return unicodeData.isSkipGeneralCategory(code, gc) ||
          code == 0x0E33 || // Thai U+0E33 is class AM: https://www.microsoft.com/typography/OpenTypeDev/thai/intro.htm
          code == 0x0EB3; // Lao U+0EB3 is class AM: https://www.microsoft.com/typography/OpenTypeDev/lao/intro.htm
      }
      if (noSkip)
        return skipCombiningMarks;
      return function (code) { return unicodeData.isCJKMiddle(code) || skipCombiningMarks(code); };
    }
    splitCodePointsByBlocks(codePoints) {
      return unicodeData.splitCodePoints(codePoints, this.blocks);
    }
    linesFromCodePoints(codePoints) {
      var lines = [];
      var limit = codePoints.length;
      for (var index = 0; index < limit; ) {
        var lineLimit = Math.min(limit, index + this.charactersPerLine);
        var line = [];
        for (; index < lineLimit; ++index)
          unicodeData.encodeUtf16(codePoints[index], line);
        lines.push(String.fromCharCode.apply(String, line));
      }
      return lines;
    }
  }

  function affixFromIndex(index) {
    if (index < 0)
      return "";
    if (index >= 26)
      throw new Error("Affix index too large (" + index + ")");
    return String.fromCharCode("a".charCodeAt(0) + index);
  }

  function createGenerator(argv) {
    var promise = new Promise(function(resolve, reject) {
      Promise.all([
        unicodeData.get(unicodeData.url.vo, unicodeData.formatAsRangesByValue),
        unicodeData.get(unicodeData.url.gc),
        unicodeData.get(unicodeData.url.blocks),
      ]).then(function (results) {
        var generator = new Generator(results[0], results[1], results[2]);
        generator.prefix = argv.prefix ? "-" + argv.prefix + "-" : "";
        resolve(generator);
      });
    });
    return promise;
  }

  exports.generate = function (argv) {
    return createGenerator(argv)
      .then(generator => generator.generate(argv));
  };

  exports.generateRefTest = function (argv) {
    return createGenerator(argv)
      .then(generator => generator.generateRefTest(argv));
  };
})(module.exports);
