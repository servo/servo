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
  // Verify that only valid keys are specified. This prevents the caller from
  // accidentally supplying 'inherited' instead of 'inherits', for example.
  if (typeof(reg) === 'object') {
    const permitted = new Set(['name', 'syntax', 'initialValue', 'inherits']);
    if (!Object.keys(reg).every(k => permitted.has(k)))
      throw new Error('generate_property: invalid parameter');
  }

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

function with_style_node(text, fn) {
  let node = document.createElement('style');
  node.textContent = text;
  try {
    document.body.append(node);
    fn(node);
  } finally {
    node.remove();
  }
}

function with_at_property(desc, fn) {
  let name = typeof(desc.name) === 'undefined' ? generate_name() : desc.name;
  let text = `@property ${name} {`;
  if (typeof(desc.syntax) !== 'undefined')
    text += `syntax:${desc.syntax};`;
  if (typeof(desc.initialValue) !== 'undefined')
    text += `initial-value:${desc.initialValue};`;
  if (typeof(desc.inherits) !== 'undefined')
    text += `inherits:${desc.inherits};`;
  text += '}';
  with_style_node(text, (node) => fn(name, node.sheet.rules[0]));
}

function test_with_at_property(desc, fn, description) {
  test(() => with_at_property(desc, fn), description);
}

function test_with_style_node(text, fn, description) {
  test(() => with_style_node(text, fn), description);
}
