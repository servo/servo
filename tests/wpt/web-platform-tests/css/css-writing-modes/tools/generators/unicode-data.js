'use strict';

module.exports = (function () {
  var fs = require("fs");
  var http = require("http");
  var path = require("path");
  var stream = require("stream");
  var url = require("url");

  var unicodeData = {
    url: {
      blocks: "http://www.unicode.org/Public/UCD/latest/ucd/Blocks.txt",
      gc: "http://www.unicode.org/Public/UCD/latest/ucd/extracted/DerivedGeneralCategory.txt",
      vo: "http://www.unicode.org/Public/vertical/revision-16/VerticalOrientation-16.txt",
    },
    get: function (source, formatter) {
      formatter = formatter || this.formatAsArray;
      var buffer = "";
      var parser = new stream.Writable();
      parser._write = function (chunk, encoding, next) {
        buffer += chunk;
        next();
      };
      var promise = new Promise(function(resolve, reject) {
        parser.on("finish", function () {
          var results = null;
          for (var line of buffer.split("\n"))
            results = unicodeData.parseLine(line, formatter, results);
          resolve(results);
        });
      });
      var basename = path.basename(url.parse(source).path);
      var local = "ucd/" + basename;
      if (fs.existsSync(local)) {
        fs.createReadStream(local)
          .pipe(parser);
      } else {
        http.get(source, function (res) {
          res.pipe(parser);
        });
      }
      return promise;
    },
    copyToLocal: function () {
      for (let key in unicodeData.url) {
        let source = unicodeData.url[key];
        let basename = path.basename(url.parse(source).path);
        let local = "ucd/" + basename;
        console.log(`Copying ${key}: ${source} to ${local}`);
        http.get(source, function (res) {
          res.pipe(fs.createWriteStream(local));
          console.log(`Done ${key}: ${source} to ${local}`);
        });
      }
    },
    parseLine: function (line, formatter, results) {
      if (!line.length || line[0] == "#")
        return results;
      var match = /([0-9A-F]+)(\.\.([0-9A-F]+))?\s*;\s*(\w+)/.exec(line);
      if (!match)
        throw new Error("Inavlid format: " + line);
      var from = parseInt(match[1], 16);
      var to = match[3] ? parseInt(match[3], 16) : from;
      var value = match[4];
      return formatter(results, from, to, value);
    },
    formatAsArray: function (results, from, to, value) {
      results = results || [];
      for (var code = from; code <= to; code++)
        results[code] = value;
      return results;
    },
    formatAsRangesByValue: function (results, from, to, value) {
      results = results || {};
      var list = results[value];
      if (!list) {
        list = [];
        results[value] = list;
      } else {
        var last = list[list.length - 1];
        if (last == from - 1) {
          list[list.length - 1] = to;
          return results;
        }
      }
      list.push(from);
      list.push(to);
      return results;
    },
    arrayFromRangesByValue: function (dict) {
      var array = [];
      for (var value in dict) {
        var ranges = dict[value];
        for (var i = 0; i < ranges.length; i += 2) {
          var to = ranges[i+1];
          for (var code = ranges[i]; code <= to; code++)
            array[code] = value;
        }
      }
      return array;
    },
    isSkipGeneralCategory: function (code, gc) {
      var gc0 = gc[code][0];
      // General Category M* and C* are omitted as they're likely to not render well
      return gc0 == "M" || gc0 == "C";
    },
    isCJKMiddle: function (code) {
      // To make tests smaller, omit some obvious ranges except the first and the last
      return code > 0x3400 && code < 0x4DB5 || // CJK Unified Ideographs Extension A
        code > 0x4E00 && code < 0x9FCC || // CJK Unified Ideographs (Han)
        code > 0xAC00 && code < 0xD7A3 || // Hangul Syllables
        code > 0x17000 && code < 0x187EC || // Tangut
        code > 0x18800 && code < 0x18AF2 || // Tangut Components
        code > 0x20000 && code < 0x2A6D6 || // CJK Unified Ideographs Extension B
        code > 0x2A700 && code < 0x2B734 || // CJK Unified Ideographs Extension C
        code > 0x2B740 && code < 0x2B81D || // CJK Unified Ideographs Extension D
        code > 0x2B820 && code < 0x2CEA1; // CJK Unified Ideographs Extension E
    },
    codePointsFromRanges: function (ranges, skipFunc) {
      var codePoints = [];
      for (var i = 0; i < ranges.length; i += 2) {
        var code = ranges[i];
        var to = ranges[i+1];
        for (; code <= to; code++) {
          if (code >= 0xD800 && code <= 0xDFFF) // Surrogate Pairs
            continue;
          if (skipFunc && skipFunc(code))
            continue;
          codePoints.push(code);
        }
      }
      return codePoints;
    },
    splitCodePoints: function (codePoints, values) {
      var results = [];
      var currentCodePoints = [];
      var currentValue = null;
      for (var code of codePoints) {
        var value = values[code];
        if (value != currentValue) {
          results.push([currentCodePoints, currentValue]);
          currentValue = value;
          currentCodePoints = [];
        }
        currentCodePoints.push(code);
      }
      if (currentCodePoints.length)
        results.push([currentCodePoints, currentValue]);
      return results.slice(1);
    },
    encodeUtf16: function (code, output) {
      if (code >= 0x10000) {
        code -= 0x10000;
        output.push(code >>> 10 & 0x3FF | 0xD800);
        code = 0xDC00 | code & 0x3FF;
      }
      output.push(code);
    },
    toHex: function (value) {
      return unicodeData.padZero(value.toString(16).toUpperCase(), 4);
    },
    padZero: function (value, digits) {
      if (value.length >= digits)
        return value;
      value = "0000" + value;
      return value.substr(value.length - digits);
    },
  };
  return unicodeData;
})();
