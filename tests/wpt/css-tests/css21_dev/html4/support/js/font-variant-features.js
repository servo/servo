
// data associated with gsubtest test font for testing font features

// prefix
gPrefix = "";

// equivalent properties
// setting prop: value should match the specific feature settings listed
//
// each of these tests evaluate whether a given feature is enabled as required
// and also whether features that shouldn't be enabled are or not.
var gPropertyData = [
  // font-variant-caps
  // valid values
  { prop: "font-variant-caps", value: "normal", features: {"smcp": 0} },
  { prop: "font-variant-caps", value: "small-caps", features: {"smcp": 1, "c2sc": 0} },
  { prop: "font-variant-caps", value: "all-small-caps", features: {"smcp": 1, "c2sc": 1, "pcap": 0} },
  { prop: "font-variant-caps", value: "petite-caps", features: {"pcap": 1, "smcp": 0} },
  { prop: "font-variant-caps", value: "all-petite-caps", features: {"c2pc": 1, "pcap": 1, "smcp": 0} },
  { prop: "font-variant-caps", value: "titling-caps", features: {"titl": 1, "smcp": 0} },
  { prop: "font-variant-caps", value: "unicase", features: {"unic": 1, "titl": 0} },

  // invalid values
  { prop: "font-variant-caps", value: "normal small-caps", features: {"smcp": 0}, invalid: true },
  { prop: "font-variant-caps", value: "small-caps potato", features: {"smcp": 0}, invalid: true },
  { prop: "font-variant-caps", value: "small-caps petite-caps", features: {"smcp": 0, "pcap": 0}, invalid: true },
  { prop: "font-variant-caps", value: "small-caps all-small-caps", features: {"smcp": 0, "c2sc": 0}, invalid: true },
  { prop: "font-variant-caps", value: "small-cap", features: {"smcp": 0}, invalid: true },

  // font-variant-east-asian
  // valid values
  { prop: "font-variant-east-asian", value: "jis78", features: {"jp78": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "jis83", features: {"jp83": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "jis90", features: {"jp90": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "jis04", features: {"jp04": 1, "jp78": 0} },
  { prop: "font-variant-east-asian", value: "simplified", features: {"smpl": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "traditional", features: {"trad": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "full-width", features: {"fwid": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "proportional-width", features: {"pwid": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "ruby", features: {"ruby": 1, "jp04": 0} },
  { prop: "font-variant-east-asian", value: "jis78 full-width", features: {"jp78": 1, "fwid": 1, "jp83": 0} },
  { prop: "font-variant-east-asian", value: "jis78 full-width ruby", features: {"jp78": 1, "fwid": 1, "jp83": 0, "ruby": 1} },
  { prop: "font-variant-east-asian", value: "simplified proportional-width", features: {"smpl": 1, "pwid": 1, "jp83": 0} },
  { prop: "font-variant-east-asian", value: "ruby simplified", features: {"ruby": 1, "smpl": 1, "trad": 0} },

  // invalid values
  { prop: "font-variant-east-asian", value: "ruby normal", features: {"ruby": 0}, invalid: true },
  { prop: "font-variant-east-asian", value: "jis90 jis04", features: {"jp90": 0, "jp04": 0}, invalid: true },
  { prop: "font-variant-east-asian", value: "simplified traditional", features: {"smpl": 0, "trad": 0}, invalid: true },
  { prop: "font-variant-east-asian", value: "full-width proportional-width", features: {"fwid": 0, "pwid": 0}, invalid: true },
  { prop: "font-variant-east-asian", value: "ruby simplified ruby", features: {"ruby": 0, "smpl": 0, "jp04": 0}, invalid: true },
  { prop: "font-variant-east-asian", value: "jis78 ruby simplified", features: {"ruby": 0, "smpl": 0, "jp78": 0}, invalid: true },

  // font-variant-ligatures
  // valid values
  { prop: "font-variant-ligatures", value: "normal", features: {"liga": 1, "dlig": 0} },
  { prop: "font-variant-ligatures", value: "common-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "no-common-ligatures", features: {"liga": 0, "clig": 0, "dlig": 0, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "discretionary-ligatures", features: {"liga": 1, "clig": 1, "dlig": 1, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "no-discretionary-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "historical-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 1, "calt": 1} },
  { prop: "font-variant-ligatures", value: "no-historical-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "contextual", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "no-contextual", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0, "calt": 0} },
  { prop: "font-variant-ligatures", value: "common-ligatures no-discretionary-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "historical-ligatures no-common-ligatures", features: {"clig": 0, "liga": 0, "dlig": 0, "hlig": 1, "calt": 1} },
  { prop: "font-variant-ligatures", value: "no-historical-ligatures discretionary-ligatures", features: {"liga": 1, "clig": 1, "dlig": 1, "hlig": 0, "calt": 1} },
  { prop: "font-variant-ligatures", value: "common-ligatures no-discretionary-ligatures historical-ligatures no-contextual", features: {"clig": 1, "dlig": 0, "hlig": 1, "liga": 1, "calt": 0} },

  // invalid values
  { prop: "font-variant-ligatures", value: "common-ligatures normal", features: {"liga": 1, "clig": 1, "dlig": 0}, invalid: true },
  { prop: "font-variant-ligatures", value: "common-ligatures no-common-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0}, invalid: true },
  { prop: "font-variant-ligatures", value: "common-ligatures common-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0}, invalid: true },
  { prop: "font-variant-ligatures", value: "no-historical-ligatures historical-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0}, invalid: true },
  { prop: "font-variant-ligatures", value: "no-contextual contextual", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0}, invalid: true },
  { prop: "font-variant-ligatures", value: "no-discretionary-ligatures discretionary-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0}, invalid: true },
  { prop: "font-variant-ligatures", value: "common-ligatures no-discretionary-ligatures no-common-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0}, invalid: true },

  // font-variant-numeric
  // valid values
  { prop: "font-variant-numeric", value: "normal", features: {"lnum": 0, "tnum": 0, "pnum": 0, "onum": 0} },
  { prop: "font-variant-numeric", value: "lining-nums", features: {"lnum": 1, "onum": 0, "pnum": 0} },
  { prop: "font-variant-numeric", value: "oldstyle-nums", features: {"lnum": 0, "onum": 1, "pnum": 0} },
  { prop: "font-variant-numeric", value: "proportional-nums", features: {"lnum": 0, "onum": 0, "pnum": 1, "tnum": 0} },
  { prop: "font-variant-numeric", value: "proportional-nums oldstyle-nums", features: {"lnum": 0, "onum": 1, "pnum": 1, "tnum": 0} },
  { prop: "font-variant-numeric", value: "tabular-nums", features: {"tnum": 1, "onum": 0, "pnum": 0} },
  { prop: "font-variant-numeric", value: "diagonal-fractions", features: {"frac": 1, "afrc": 0, "pnum": 0} },
  { prop: "font-variant-numeric", value: "stacked-fractions", features: {"frac": 0, "afrc": 1, "pnum": 0} },
  { prop: "font-variant-numeric", value: "slashed-zero", features: {"zero": 1, "pnum": 0} },
  { prop: "font-variant-numeric", value: "ordinal", features: {"ordn": 1, "pnum": 0} },
  { prop: "font-variant-numeric", value: "lining-nums diagonal-fractions", features: {"frac": 1, "afrc": 0, "lnum": 1} },
  { prop: "font-variant-numeric", value: "tabular-nums stacked-fractions", features: {"frac": 0, "afrc": 1, "tnum": 1} },
  { prop: "font-variant-numeric", value: "tabular-nums slashed-zero stacked-fractions", features: {"frac": 0, "afrc": 1, "tnum": 1, "zero": 1} },
  { prop: "font-variant-numeric", value: "proportional-nums slashed-zero diagonal-fractions oldstyle-nums ordinal", features: {"frac": 1, "afrc": 0, "tnum": 0, "pnum": 1, "onum": 1, "ordn": 1, "zero": 1} },

  // invalid values
  { prop: "font-variant-numeric", value: "lining-nums normal", features: {"lnum": 0, "onum": 0}, invalid: true },
  { prop: "font-variant-numeric", value: "lining-nums oldstyle-nums", features: {"lnum": 0, "onum": 0}, invalid: true },
  { prop: "font-variant-numeric", value: "lining-nums normal slashed-zero ordinal", features: {"lnum": 0, "onum": 0, "zero": 0}, invalid: true },
  { prop: "font-variant-numeric", value: "proportional-nums tabular-nums", features: {"pnum": 0, "tnum": 0}, invalid: true },
  { prop: "font-variant-numeric", value: "diagonal-fractions stacked-fractions", features: {"frac": 0, "afrc": 0}, invalid: true },
  { prop: "font-variant-numeric", value: "slashed-zero diagonal-fractions slashed-zero", features: {"frac": 0, "afrc": 0, "zero": 0}, invalid: true },
  { prop: "font-variant-numeric", value: "lining-nums slashed-zero diagonal-fractions oldstyle-nums", features: {"frac": 0, "afrc": 0, "zero": 0, "onum": 0}, invalid: true },

  // font-variant-position
  // valid values
  { prop: "font-variant-position", value: "normal", features: {"subs": 0, "sups": 0} },
  { prop: "font-variant-position", value: "super", features: {"subs": 0, "sups": 1} },
  { prop: "font-variant-position", value: "sub", features: {"subs": 1, "sups": 0} },

  // invalid values
  { prop: "font-variant-position", value: "super sub", features: {"subs": 0, "sups": 0}, invalid: true },
];

// note: the code below requires an array "gFeatures" from :
//   support/fonts/gsubtest-features.js

// The font defines feature lookups for all OpenType features for a
// specific set of PUA codepoints, as listed in the gFeatures array.
// Using these codepoints and feature combinations, tests can be
// constructed to detect when certain features are enabled or not.

// return a created table containing tests for a given property
//
// Ex: { prop: "font-variant-ligatures", value: "common-ligatures", features: {"liga": 1, "clig": 1, "dlig": 0, "hlig": 0} }
//
// This means that for the property 'font-variant-ligatures' with the value 'common-ligatures', the features listed should
// either be explicitly enabled or disabled.

// propData is the prop/value list with corresponding feature assertions
// whichProp is either "all" or a specific subproperty (i.e. "font-variant-position")
// isRef is true when this is the reference
// debug outputs the prop/value pair along with the tests

function createFeatureTestTable(propData, whichProp, isRef, debug)
{
  var table = document.createElement("table");

  if (typeof(isRef) == "undefined") {
    isRef = false;
  }

  if (typeof(debug) == "undefined") {
    debug = false;
  }

  var doAll = (whichProp == "all");
  for (var i in propData) {
    var data = propData[i];

    if (!doAll && data.prop != whichProp) continue;

    var row = document.createElement("tr");
    var invalid = false;
    if ("invalid" in data) {
      invalid = true;
      row.className = "invalid";
    }

    var cell = document.createElement("td");
    cell.className = "prop";
    var styledecl = gPrefix + data.prop + ": " + data.value + ";";
    cell.innerHTML = styledecl;
    row.appendChild(cell);
    if (debug) {
      table.appendChild(row);
    }

    row = document.createElement("tr");
    if (invalid) {
      row.className = "invalid";
    }

    cell = document.createElement("td");
    cell.className = "features";
    if (!isRef) {
      cell.style.cssText = styledecl;
    }

    for (var f in data.features) {
      var feature = data.features[f];

      var cp, unsupported = "F".charCodeAt(0);
      var basecp = gFeatures[f];

      if (typeof(basecp) == "undefined") {
        cp = unsupported;
      } else {
        switch(feature) {
        case 0:
          cp = basecp;
          break;
        case 1:
          cp = basecp + 1;
          break;
        case 2:
          cp = basecp + 2;
          break;
        case 3:
          cp = basecp + 3;
          break;
        default:
          cp = basecp + 1;
          break;
        }
      }

      var span = document.createElement("span");
      span.innerHTML = (isRef ? "P " : "&#x" + cp.toString(16) + "; ");
      span.title = f + "=" + feature;
      cell.appendChild(span);
    }
    row.appendChild(cell);
    table.appendChild(row);
  }

  return table;
}


