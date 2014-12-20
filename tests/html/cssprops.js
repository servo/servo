function run_tests(properties) {
  for (var property in Object.keys(properties)) {
    var name = Object.keys(properties)[property];
    var generator = create_value_generator(properties[name]);
    var prop = properties[name].property || name;
    while (run_test(name, generator, prop)) {
    }
  }
}

function generate_inline_style(name, value) {
  if (value) {
    return {'declaration': name + ": " + value,
            'value': value,
            'result': value};
  }
  return null;
}

function all_values(values) {
  var results = [];
  for (var i = 0; i < values.length; i++) {
    var value = values[i];
    if (typeof value == "function") {
      var f = value();
      var result;
      while ((result = f()) != null) {
        if (typeof result == "object" && 'serialized' in result) {
          results.push(result.serialized); //XXXjdm push actual and expect serialized
        } else {
          results.push(result);
        }
      }
    } else if (typeof value == "string") {
      results.push(value);
    } else if (value instanceof Array) {
      var subresults = [];
      for (var j = 0; j < value.length; j++) {
	var subresult = all_values(value[j], true);
        if (!(subresult instanceof Array)) {
	    subresult = [subresult];
	}
        subresults.push(subresult);
      }
      if (subresults.length > 1) {
        function choose_slices(vecs) {
          if (vecs.length == 1) {
            return vecs[0].map(function(v) { return [v]; });
          }
	  var slice_results = [];
          var rest = choose_slices(vecs.slice(1, vecs.length));
          for (var a = 0; a < vecs[0].length; a++) {
            for (var b = 0; b < rest.length; b++) {
              slice_results.push([vecs[0][a]].concat(rest[b]));
            }
          }
          return slice_results;
        }

        subresults = choose_slices(subresults).map(function (a) { return a.join(' ') });
      }
      for (var j = 0; j < subresults.length; j++) {
        results = results.concat(subresults[j]);
      }
    } else if (value instanceof Object && 'serialized' in value) {
      results.push(value.serialized); //XXXjdm push actual and expect serialized
    } else if (typeof value == "number") {
      results.push(value.toString());
    } else {
      throw "unexpected value type: " + typeof(value);
    }
  }
  return results;
}

function create_value_generator(property) {
  var results = all_values(property.values);
  return iterable(results);
}

function to_idl(property) {
  return property.replace(/-\w/g, function(x){return x.toUpperCase()}).split('-').join('');
}

function run_test(property, generator, prop) {
  var elem = document.createElement('div');
  document.getElementById('parent').appendChild(elem);
  var style = generate_inline_style(property, generator());
  if (style && to_idl(prop) in elem.style) {
    elem.setAttribute('style', style.declaration);
    is(elem.style[to_idl(prop)], style.result, property + ' raw inline style declaration');
    elem.setAttribute('style', '');
    elem.style[to_idl(prop)] = style.value;
    is(elem.style[to_idl(prop)], style.result, property + ' style property');
  }
  document.getElementById('parent').removeChild(elem);
  return style != null;
}

function iterable(values) {
  var i = 0;
  return function() {
    if (i < values.length) {
      return values[i++];
    }
    return null;
  }
}

function color() {
  var colors = ['black', 'red', 'rgb(50, 75, 100)', 'rgba(5, 7, 10, 0.9)'];
  return iterable(colors);
}

function percentage() {
  var values = ["5%", {actual: ".5%", serialized: "0.5%"}];
  return iterable(values);
}

function length() {
  var values = ["1px", {actual: ".1em", serialized: "0.1em"}];
  return iterable(values);
}

function degree() {
  var values = ["87deg"];
  return iterable(values);
}

function uri() {
  var values = ["url(\"http://localhost/\")",
                {actual: "url(http://localhost/)",
                 serialized: "url(\"http://localhost/\")"}];
  return iterable(values);
}

function border_style() {
  var values = ['none', 'hidden', 'dotted', 'dashed', 'solid', 'double', 'groove', 'ridge',
                'inset', 'outset'];
  return iterable(values);
}

function integer() {
  var values = ['0', '101', '-51'];
  return iterable(values);
}

function shape() {
  var values = ['rect(1em, auto, 0.5px, 2000em)'];
  return iterable(values);
}

function string() {
  var values = ['"string"', {actual: "'string'", serialized: '"string"'}];
  return iterable(values);
}

function counter() {
  var values = ['counter(par-num)', 'counter(par-num, upper-roman)'];
  return iterable(values);
}

function attr() {
  var values = ['attr(foo-bar)', 'attr(foo_bar)'];
  return iterable(values);
}

function family_name() {
  var values = ['Gill,', '"Lucida" Grande,', 'Red/Black,'];
  return iterable(values);
}

function generic_family() {
  var values = ['serif', 'sans-serif'];
  return iterable(values);
}

function absolute_size() {
  var values = ['xx-small', 'x-small', 'small', 'medium', 'large', 'x-large', 'xx-large'];
  return iterable(values);
}

function relative_size() {
  var values = ['larger', 'smaller'];
  return iterable(values);
}

function number() {
  var values = ['0', {'actual': '-0', serialized: '0'}, '1000', '-5123', '0.9', '-0.09'];
  return iterable(values);
}

var properties = {
  'background-attachment': {
    'values': ['scroll', 'fixed', 'inherit'],
    'initial': 'scroll',
  },
  'background-color': {
    'values': [color, 'transparent', 'inherit'],
    'initial': 'transparent',
  },
  'background-image': {
    'values': [uri, 'none', 'inherit'],
    'initial': 'none',
  },
  'background-position': {
    'values': [[[percentage, length, 'left', 'center', 'right'],
                [percentage, length, 'top', 'center', 'bottom']],
               [['left', 'center', 'right'],
                ['top', 'center', 'bottom']],
                'inherit'],
    'initial': '0% 0%',
  },
  'background-repeat': {
    'values': ['repeat', 'repeat-x', 'repeat-y', 'no-repeat', 'inherit'],
    'initial': 'repeat',
  },
  //background
  'border-collapse': {
    'values': ['collapse', 'separate', 'inherit'],
    'initial': 'separate',
  },
  //border-color
  'border-spacing': {
    'values': [length, 'inherit'],
    'initial': '0',
  },
  //border-style
  //border-top, border-right, border-bottom, border-left
  'border-top-color': {
    'values': [color, 'transparent', 'inherit'],
    'initial': 'black', //FIXME
  },
  'border-right-color': {
    'values': [color, 'transparent', 'inherit'],
    'initial': 'black', //FIXME
  },
  'border-bottom-color': {
    'values': [color, 'transparent', 'inherit'],
    'initial': 'black', //FIXME
  },
  'border-left-color': {
    'values': [color, 'transparent', 'inherit'],
    'initial': 'black', //FIXME
  },
  'border-top-style': {
    'values': [border_style, 'inherit'],
    'initial': null,
  },
  'border-right-style': {
    'values': [border_style, 'inherit'],
    'initial': null,
  },
  'border-bottom-style': {
    'values': [border_style, 'inherit'],
    'initial': null,
  },
  'border-left-style': {
    'values': [border_style, 'inherit'],
    'initial': null,
  },
  'border-top-width': {
    'values': ['thin', 'medium', 'thick', length, 'inherit'],
    'initial': 'medium',
  },
  'border-right-width': {
    'values': ['thin', 'medium', 'thick', length, 'inherit'],
    'initial': 'medium',
  },
  'border-bottom-width': {
    'values': ['thin', 'medium', 'thick', length, 'inherit'],
    'initial': 'medium',
  },
  'border-left-width': {
    'values': ['thin', 'medium', 'thick', length, 'inherit'],
    'initial': 'medium',
  },
  //border-width
  //border
  'bottom': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'caption-side': {
    'values': ['top', 'bottom', 'inherit'],
    'initial': 'top',
  },
  'clear': {
    'values': ['none', 'left', 'right', 'both', 'inherit'],
    'initial': 'none',
  },
  'clip': {
    'values': [shape, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'color': {
    'values': [color, 'inherit'],
    'initial': 'black', //FIXME depends on user agent
  },
  'content': {
    'values': ['normal', 'none', string, uri, counter, attr, 'inherit'], //FIXME
    'initial': 'normal',
  },
  //counter-increment
  //counter-reset
  'cursor': {
    'values': [/*uri,*/ 'auto', 'crosshair', 'default', 'pointer', 'move', 'e-resize', 'ne-resize',
               'nw-resize', 'n-resize', 'se-resize', 'sw-resize', 's-resize', 'w-resize', 'text',
               'wait', 'help', 'progress', 'inherit'],
    'initial': 'auto',
  },
  'direction': {
    'values': ['ltr', 'rtl', 'inherit'],
    'initial': 'ltr',
  },
  'display': {
    'values': ['inline', 'block', 'list-item', 'inline-block', 'table', 'inline-table',
               'table-row-group', 'table-header-group', 'table-footer-group', 'table-row',
               'table-column-group', 'table-column', 'table-cell', 'table-caption', 'none',
               'inherit'],
    'initial': 'inline',
  },
  'empty-cells': {
     'values': ['show', 'hide', 'inherit'],
     'initial': 'show',
  },
  'float': {
    'values': ['left', 'right', 'none', 'inherit'],
    'initial': 'none',
    'property': 'cssFloat',
  },
  'font-family': {
    'values': [[family_name, generic_family], 'inherit'],
    'initial': 'sans-serif', //FIXME depends on user agent
  },
  'font-size': {
    'values': [absolute_size, relative_size, length, percentage, 'inherit'],
    'initial': 'medium',
  },
  'font-style': {
    'values': ['normal', 'italic', 'oblique', 'inherit'],
    'initial': 'normal',
  },
  'font-variant': {
    'values': ['normal', 'small-caps', 'inherit'],
    'initial': 'normal',
  },
  'font-weight': {
    'values': ['normal', 'bold', 'bolder', 'lighter', 100, 200, 300, 300, 400, 500, 600,
               700, 800, 900, 'inherit'],
    'initial': 'normal',
  },
  //font
  'height': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'left': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'letter-spacing': {
    'values': ['normal', length, 'inherit'],
    'initial': 'normal',
  },
  'line-height': {
    'values': ['normal', number, length, percentage, 'inherit'],
    'initial': 'normal',
  },
  'list-style-image': {
    'values': [uri, 'none', 'inherit'],
    'initial': 'none',
  },
  'list-style-position': {
    'values': ['inside', 'outside', 'inherit'],
    'initial': 'outside',
  },
  'list-style-type': {
    'values': ['disc', 'circle', 'square', 'decimal', 'decimal-leading-zero', 'lower-roman',
               'upper-roman', 'lower-greek', 'lower-latin', 'upper-latin', 'armenian', 'georgian',
               'lower-alpha', 'upper-alpha', 'none', 'inherit'],
    'initial': 'disc',
  },
  //list-style
  'margin-right': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 0,
  },
  'margin-left': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 0,
  },
  'margin-top': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 0,
  },
  'margin-bottom': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 0,
  },
  //margin
  'max-height': {
    'values': [length, percentage, 'none', 'inherit'],
    'initial': 'none',
  },
  'max-width': {
    'values': [length, percentage, 'none', 'inherit'],
    'initial': 'none',
  },
  'min-height': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  'min-width': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  'orphans': {
    'values': [integer, 'inherit'],
    'initial': 2,
  },
  'outline-color': {
    'values': [color, 'invert', 'inherit'],
    'initial': 'invert',
  },
  'outline-style': {
    'values': [border_style, 'inherit'],
    'initial': 'none',
  },
  'outline-width': {
    'values': ['thin', 'medium', 'thick', length, 'inherit'],
    'initial': 'medium',
  },
  //outline
  'overflow': {
    'values': ['visible', 'hidden', 'scroll', 'auto', 'inherit'],
    'initial': 'visible',
  },
  'padding-top': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  'padding-right': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  'padding-bottom': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  'padding-left': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  //padding
  'page-break-after': {
    'values': ['auto', 'always', 'avoid', 'left', 'right', 'inherit'],
    'initial': 'auto',
  },
  'page-break-before': {
    'values': ['auto', 'always', 'avoid', 'left', 'right', 'inherit'],
    'initial': 'auto',
  },
  'page-break-inside': {
    'values': ['avoid', 'auto', 'inherit'],
    'initial': 'auto',
  },
  'position': {
    'values': ['static', 'relative', 'absolute', 'fixed', 'inherit'],
    'initial': 'static',
  },
  //FIXME quotes
  'right': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'table-layout': {
    'values': ['auto', 'fixed', 'inherit'],
    'initial': 'auto',
  },
  'text-align': {
    'values': ['left', 'right', 'center', 'justify', 'inherit'],
    'initial': null,
  },
  'text-decoration': {
    'values': ['none', 'underline', 'overline', 'line-through', 'blink', 'inherit'],
    'initial': 'none',
  },
  'text-indent': {
    'values': [length, percentage, 'inherit'],
    'initial': 0,
  },
  'text-transform': {
    'values': ['capitalize', 'uppercase', 'lowercase', 'none', 'inherit'],
    'initial': 'none',
  },
  'top': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'unicode-bidi': {
    'values': ['normal', 'embed', 'bidi-override', 'inherit'],
    'initial': 'normal',
  },
  'vertical-align': {
    'values': ['baseline', 'sub', 'super', 'top', 'text-top', 'middle', 'bottom', 'text-bottom',
               percentage, length, 'inherit'],
    'initial': 'baseline',
  },
  'visibility': {
    'values': ['visible', 'hidden', 'collapse', 'inherit'],
    'initial': 'visible',
  },
  'white-space': {
    'values': ['normal', 'pre', 'nowrap', 'pre-wrap', 'pre-line', 'inherit'],
    'initial': 'normal',
  },
  'widows': {
    'values': [integer, 'inherit'],
    'initial': 2,
  },
  'width': {
    'values': [length, percentage, 'auto', 'inherit'],
    'initial': 'auto',
  },
  'word-spacing': {
    'values': ['normal', length, 'inherit'],
    'initial': 'normal',
  },
  'z-index': {
    'values': ['auto', integer, 'inherit'],
    'initial': 'auto',
  },
};
