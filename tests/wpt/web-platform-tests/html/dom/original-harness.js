var ReflectionHarness = {};

// @private
ReflectionHarness.passed = document.getElementById("passed");
ReflectionHarness.failed = document.getElementById("failed");

/**
 * Should we report a failure for unexpected exceptions, or just rethrow them?
 * The original test framework reports an exception, but testharness.js doesn't
 * want that.
 *
 * @public
 */
ReflectionHarness.catchUnexpectedExceptions = true;

/**
 * Returns a string representing val.  Basically just adds quotes for strings,
 * and passes through other recognized types literally.
 *
 * @public
 */
ReflectionHarness.stringRep = function(val) {
  if (val === null) {
    // typeof is object, so the switch isn't useful
    return "null";
  }
  // In JavaScript, -0 === 0 and String(-0) == "0", so we have to
  // special-case.
  if (val === -0 && 1/val === -Infinity) {
    return "-0";
  }
  switch (typeof val) {
    case "string":
      for (var i = 0; i < 32; i++) {
        var replace = "\\";
        switch (i) {
          case 0: replace += "0"; break;
          case 1: replace += "x01"; break;
          case 2: replace += "x02"; break;
          case 3: replace += "x03"; break;
          case 4: replace += "x04"; break;
          case 5: replace += "x05"; break;
          case 6: replace += "x06"; break;
          case 7: replace += "x07"; break;
          case 8: replace += "b"; break;
          case 9: replace += "t"; break;
          case 10: replace += "n"; break;
          case 11: replace += "v"; break;
          case 12: replace += "f"; break;
          case 13: replace += "r"; break;
          case 14: replace += "x0e"; break;
          case 15: replace += "x0f"; break;
          case 16: replace += "x10"; break;
          case 17: replace += "x11"; break;
          case 18: replace += "x12"; break;
          case 19: replace += "x13"; break;
          case 20: replace += "x14"; break;
          case 21: replace += "x15"; break;
          case 22: replace += "x16"; break;
          case 23: replace += "x17"; break;
          case 24: replace += "x18"; break;
          case 25: replace += "x19"; break;
          case 26: replace += "x1a"; break;
          case 27: replace += "x1b"; break;
          case 28: replace += "x1c"; break;
          case 29: replace += "x1d"; break;
          case 30: replace += "x1e"; break;
          case 31: replace += "x1f"; break;
        }
        val = val.replace(String.fromCharCode(i), replace);
      }
      return '"' + val.replace('"', '\\"') + '"';
    case "boolean":
    case "undefined":
    case "number":
      return val + "";
    default:
      return typeof val + ' "' + val + '"';
  }
}

/**
 * An object representing info about the current test, used for printing out
 * nice messages and so forth.
 */
ReflectionHarness.currentTestInfo = {};

/**
 * This is called when we want to test a single element/attribute combination.
 * For the original harness, it does nothing special (just calls the function),
 * but for testharness.js, it can wrap everything in a test() call.
 */
ReflectionHarness.testWrapper = function(fn) {
  fn();
}

/**
 * If question === answer, output a success, else report a failure with the
 * given description.  Currently success and failure both increment counters,
 * and failures output a message to a <ul>.  Which <ul> is decided by the type
 * parameter -- different attribute types are separated for readability.
 *
 * @public
 */
ReflectionHarness.test = function(expected, actual, description) {
  // Special-case -0 yay!
  if (expected === 0 && actual === 0 && 1/expected === 1/actual) {
    this.increment(this.passed);
    return true;
  } else if (expected === actual) {
    this.increment(this.passed);
    return true;
  } else {
    this.increment(this.failed);
    this.reportFailure(description + ' (expected ' + this.stringRep(actual) + ', got ' + this.stringRep(expected) + ')');
    return false;
  }
}

ReflectionHarness.run = function(fun, description) {
  try {
    fun();
  } catch (err) {
    ReflectionHarness.failure(description);
  }
}

/**
 * If calling fn causes a DOMException of the type given by the string
 * exceptionName (e.g., "INDEX_SIZE_ERR"), output a success.  Otherwise, report
 * a failure with the given description.
 *
 * @public
 */
ReflectionHarness.testException = function(exceptionName, fn, description) {
  try {
    fn();
  } catch (e) {
    if (e instanceof DOMException && e.code == DOMException[exceptionName]) {
      this.increment(this.passed);
      return true;
    }
  }
  this.increment(this.failed);
  this.reportFailure(description);
  return false;
}

/**
 * Get a description of the current type, e.g., "a.href".
 */
ReflectionHarness.getTypeDescription = function() {
  var domNode = this.currentTestInfo.domObj.tagName.toLowerCase();
  var idlNode = this.currentTestInfo.idlObj.nodeName.toLowerCase();
  var domName = this.currentTestInfo.domName;
  var idlName = this.currentTestInfo.idlName;
  var comment = this.currentTestInfo.data.comment;
  var typeDesc = idlNode + "." + idlName;
  if (!comment && (domNode != idlNode || domName != idlName)) {
    comment = "<" + domNode + " " + domName + ">";
  }
  if (comment) {
    typeDesc += " (" + comment + ")";
  }
  return typeDesc;
}

/**
 * Report a failure with the given description, adding context from the
 * currentTestInfo member.
 *
 * @private
 */
ReflectionHarness.reportFailure = function(description) {
  var typeDesc = this.getTypeDescription();
  var idlName = this.currentTestInfo.idlName;
  var comment = this.currentTestInfo.data.comment;
  typeDesc = typeDesc.replace("&", "&amp;").replace("<", "&lt;");
  description = description.replace("&", "&amp;").replace("<", "&lt;");

  var type = this.currentTestInfo.data.type;

  // Special case for undefined attributes, which we don't want getting in
  // the way of everything else.
  if (description.search('^typeof IDL attribute \\(expected ".*", got "undefined"\\)$') != -1) {
    type = "undefined";
  }

  var done = false;
  var ul = document.getElementById("errors-" + type.replace(" ", "-"));
  if (ul === null) {
    ul = document.createElement("ul");
    ul.id = "errors-" + type.replace(" ", "-");
    var div = document.getElementById("errors");
    p = document.createElement("p");
    if (type == "undefined") {
      div.parentNode.insertBefore(ul, div.nextSibling);
      p.innerHTML = "These IDL attributes were of undefined type, presumably representing unimplemented features (cordoned off into a separate section for tidiness):";
    } else {
      div.appendChild(ul);
      p.innerHTML = "Errors for type " + type + ":";
    }
    ul.parentNode.insertBefore(p, ul);
  } else if (type != "undefined") {
    var existingErrors = ul.getElementsByClassName("desc");
    for (var i = 0; i < existingErrors.length; i++) {
      if (existingErrors[i].innerHTML == description) {
        var typeSpan = existingErrors[i].parentNode.getElementsByClassName("type")[0];
        // Check if we have lots of the same error for the same
        // attribute.  If so, we want to collapse them -- the exact
        // elements that exhibit the error aren't going to be important
        // to report in this case, and it can take a lot of space if
        // there's an error in a global attribute like dir or id.
        var types = typeSpan.innerHTML.split(", ");
        var count = 0;
        for (var i = 0; i < types.length; i++) {
          if (types[i].search("^\\([0-9]* elements\\)\\." + idlName + "$") != -1) {
            types[i] = "(" + (1 + parseInt(/[0-9]+/.exec(types[i])[0])) + " elements)." + idlName;
            typeSpan.innerHTML = types.join(", ");
            return;
          } else if (types[i].search("\\." + idlName + "$") != -1) {
            count++;
          }
        }
        if (comment || count < 10) {
          // Just add the extra error to the end, not many duplicates
          // (or we have a comment)
          typeSpan.innerHTML += ", " + typeDesc;
        } else {
          var filteredTypes = types.filter(function(type) { return type.search("\\." + idlName + "$") == -1; });
          if (filteredTypes.length) {
            typeSpan.innerHTML = filteredTypes.join(", ") + ", ";
          } else {
            typeSpan.innerHTML = "";
          }
          typeSpan.innerHTML += "(" + (types.length - filteredTypes.length) + " elements)." + idlName;
        }
        return;
      }
    }
  }

  if (type == "undefined") {
    ul.innerHTML += "<li>" + typeDesc;
  } else {
    ul.innerHTML += "<li><span class=\"type\">" + typeDesc + "</span>: <span class=\"desc\">" + description + "</span>";
  }
}

/**
 * Shorthand function for when we have a failure outside of test().  Generally
 * used when the failure is an exception thrown unexpectedly or such, something
 * not equality-based.
 *
 * @public
 */
ReflectionHarness.failure = function(message) {
  this.increment(this.failed);
  this.reportFailure(message);
}

/**
 * Shorthand function for when we have a success outside of test().  Only
 * called if catchUnexpectedExceptions is true.
 *
 * @public
 */
ReflectionHarness.success = function() {
  this.increment(this.passed);
}

/**
 * Increment the count in either "passed" or "failed".  el should always be one
 * of those two variables.  The implementation of this function amuses me.
 *
 * @private
 */
ReflectionHarness.increment = function(el) {
  el.innerHTML = parseInt(el.innerHTML) + 1;
  var percent = document.getElementById("percent");
  var passed = document.getElementById("passed");
  var failed = document.getElementById("failed");
  percent.innerHTML = (parseInt(passed.innerHTML)/(parseInt(passed.innerHTML) + parseInt(failed.innerHTML))*100).toPrecision(3);
}

/**
 * Hide all displayed errors matching a given regex, so it's easier to filter
 * out repetitive failures.  TODO: Fix this so it works right with the new
 * "lump many errors in one <li>" thing.
 *
 * @private (kind of, only called in the original reflection.html)
 */
ReflectionHarness.maskErrors = function(regex) {
  var uls = document.getElementsByTagName("ul");
  for (var i = 0; i < uls.length; i++) {
    var lis = uls[i].children;
    for (var j = 0; j < lis.length; j++) {
      if (regex !== "" && lis[j].innerHTML.match(regex)) {
        lis[j].style.display = "none";
      } else {
        lis[j].style.display = "list-item";
      }
    }
  }
}

// Now for some stuff that has nothing to do with ReflectionHarness and
// everything to do with initialization needed for reflection.js, which seems
// pointless to put in an extra file.

var elements = {};

var extraTests = [];

/**
 * Used for combining a number of small arrays of element data into one big
 * one.
 */
function mergeElements(src) {
  for (var key in src) {
    if (!src.hasOwnProperty(key)) {
      // This is inherited from a prototype or something.
      continue;
    }

    if (key in elements) {
      elements[key] = elements[key].concat(src[key]);
    } else {
      elements[key] = src[key];
    }
  }
}
