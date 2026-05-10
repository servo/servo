test(() => {
  // Valid names
  const validNames = [
    'a',
    'A',
    '0',
    'valid-name',
    'valid_name',
    'valid.name',
    'valid-name.with_extras-123',
    'a'.repeat(128)
  ];

  for (const name of validNames) {
    navigator.modelContext.registerTool({
      name: name,
      description: 'valid name test',
      execute: async () => 'empty'
    });
  }
}, 'Valid tool names are accepted.');

test(() => {
  // Invalid names
  const invalidNames = [
    '', // Empty
    'a'.repeat(129), // Too long
    'name with space',
    'name@special',
    'name#special',
    'name$special',
    'name%',
    'name^',
    'name&',
    'name*',
    'name(',
    'name)',
    'name+',
    'name=',
    'name[',
    'name]',
    'name{',
    'name}',
    'name|',
    'name\\',
    'name:',
    'name;',
    'name"',
    'name\'',
    'name<',
    'name>',
    'name?',
    'name/',
    'name`',
    'name~',
  ];

  for (const name of invalidNames) {
    assert_throws_dom(
      'InvalidStateError',
      () => {
        navigator.modelContext.registerTool({
          name: name,
          description: 'invalid name test',
          execute: async () => 'empty'
        });
      },
      `Tool name '${name}' is invalid.`
    );
  }
}, 'Invalid tool names throw InvalidStateError.');
