let next_property_id = 1;

// Generate a unique property name on the form --prop-N.
function generate_name() {
  return `--prop-${next_property_id++}`;
}

// Produce a compatible initial value for the specified syntax.
function any_initial_value(syntax) {
  let components = syntax.split('|').map(x => x.trim())
  let first_component = components[0];

  if (first_component.endsWith('+') || first_component.endsWith('#'))
    first_component = first_component.slice(0, -1);

  switch (first_component) {
    case '*':
    case '<custom-ident>':
      return 'NULL';
    case '<angle>':
      return '0deg';
    case '<color>':
      return 'rgb(0, 0, 0)';
    case '<image>':
    case '<url>':
      return 'url(0)';
    case '<integer>':
    case '<length-percentage>':
    case '<length>':
    case '<number>':
      return '0';
    case '<percentage>':
      return '0%';
    case '<resolution>':
      return '0dpi';
    case '<time>':
      return '0s';
    case '<transform-function>':
    case '<transform-list>':
      return 'matrix(0, 0, 0, 0, 0, 0)';
    default:
      // We assume syntax is a specific custom ident.
      return first_component;
  }
}

// Registers a unique property on the form '--prop-N' and returns the name.
// Any value except 'syntax' may be omitted, in which case the property will
// not inherit, and some undefined (but compatible) initial value will be
// generated. If a single string is used as the argument, it is assumed to be
// the syntax.
function generate_property(reg) {
  let syntax = typeof(reg) === 'string' ? reg : reg.syntax;
  let initial = typeof(reg.initialValue) === 'undefined' ? any_initial_value(syntax)
                                                         : reg.initialValue;
  let inherits = typeof(reg.inherits) === 'undefined' ? false : reg.inherits;

  let name = generate_name();
  CSS.registerProperty({
    name: name,
    syntax: syntax,
    initialValue: initial,
    inherits: inherits
  });
  return name;
}

function all_syntaxes() {
  return [
    '*',
    '<angle>',
    '<color>',
    '<custom-ident>',
    '<image>',
    '<integer>',
    '<length-percentage>',
    '<length>',
    '<number>',
    '<percentage>',
    '<resolution>',
    '<time>',
    '<transform-function>',
    '<transform-list>',
    '<url>'
  ]
}
