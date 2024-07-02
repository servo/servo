function roundNumbers(value, digits) {
  // Round numbers to |digits| decimal places.
  return value.
    replace(/-?\d*\.\d+(e-?\d+)?/g, function(n) {
      return (parseFloat(n).toFixed(digits)).
        replace(/\.\d+/, function(m) {
          return m.replace(/0+$/, '');
        }).
        replace(/\.$/, '').
        replace(/^-0$/, '0');
    });
}

function normalizeValue(value, digits) {
  // Round numbers and place whitespace between tokens.
  return roundNumbers(value, digits).
    replace(/([\w\d.]+|[^\s])/g, '$1 ').
    replace(/\s+/g, ' ');
}

// Transform a path seg list into a path string, rounding numbers to |digits|
// decimal places.
function serializePathSegList(list, digits) {
  function segmentArguments(segment) {
    const kCommandDescriptor = {
      'M': ['x', 'y'],
      'L': ['x', 'y'],
      'C': ['x1', 'y1', 'x2', 'y2', 'x', 'y'],
      'Q': ['x1', 'y1', 'x', 'y'],
      'S': ['x2', 'y2', 'x', 'y'],
      'T': ['x', 'y'],
      'A': ['r1', 'r2', 'angle', 'largeArcFlag', 'sweepFlag', 'x', 'y'],
      'H': ['x'],
      'V': ['y'],
      'Z': []
    };
    let command = segment.pathSegTypeAsLetter.toUpperCase();
    return kCommandDescriptor[command].map(field => {
      return Number(segment[field]).toFixed(digits);
    });
  }
  return Array.from(list).map(segment => {
    let command = segment.pathSegTypeAsLetter;
    if (command === 'z')
      command = 'Z';
    return [command, ...segmentArguments(segment)].join(' ');
  }).join(' ');
}

function normalizeProperty(path_string) {
  let probePathElement = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  probePathElement.setAttribute('d', path_string);
  document.documentElement.appendChild(probePathElement);
  let string = getComputedStyle(probePathElement).getPropertyValue('d');
  probePathElement.remove();
  return string;
}

// Assert that the animated path data of |target| matches one of
// |expected_paths|. Numbers will be rounded to 2 decimal places.
function assert_animated_path_in_array(target, expected_paths) {
  const kDecimals = 2;
  let expected, actual;
  if ('animatedPathSegList' in target) {
    let probePathElement = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    expected = expected_paths.map(p => {
      probePathElement.setAttribute('d', p);
      return serializePathSegList(probePathElement.pathSegList, kDecimals)
    });
    actual = serializePathSegList(target.animatedPathSegList, kDecimals);
  } else if ('d' in target.style) {
    expected = expected_paths.map(p => normalizeValue(normalizeProperty(p), kDecimals));
    actual = normalizeValue(getComputedStyle(target).getPropertyValue('d'), kDecimals);
  } else {
    assert_unreached('no animated path data');
  }
  assert_in_array(actual, expected);
}

// Assert that the animated path data of |target| matches that of
// |expected_path_string|. Numbers will be rounded to 2 decimal places.
function assert_animated_path_equals(target, expected_path_string) {
  return assert_animated_path_in_array(target, [expected_path_string]);
}
