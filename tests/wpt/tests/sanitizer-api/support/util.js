function is_same_sanitizer_name(a, b) {
  return a.name === b.name && a.namespace === b.namespace;
}

// https://pr-preview.s3.amazonaws.com/otherdaniel/purification/pull/296.html#sanitizerconfig-valid
function assert_config_is_valid(config) {
  // The config has either an elements or a removeElements key, but not both.
  assert_false(
    "elements" in config && "removeElements" in config,
    "Either elements or a removeElements, but not both",
  );
  assert_true(
    "elements" in config || "removeElements" in config,
    "Either elements or a removeElements",
  );

  // The config has either an attributes or a removeAttributes key, but not both.
  assert_false(
    "attributes" in config && "removeAttributes" in config,
    "Either attributes or a removeAttributes, but not both",
  );
  assert_true(
    "attributes" in config || "removeAttributes" in config,
    "Either attributes or removeAttributes",
  );

  // If both config[elements] and config[replaceWithChildrenElements] exist, then the difference of config[elements] and config[replaceWithChildrenElements] is empty.
  if (config.elements && config.replaceWithChildrenElements) {
    for (let element of config.elements) {
      assert_false(
        config.replaceWithChildrenElements.some((replaceElement) =>
          is_same_sanitizer_name(element, replaceElement),
        ),
        `replaceWithChildrenElements should not contain ${element.name}`,
      );
    }
  }

  // If both config[removeElements] and config[replaceWithChildrenElements] exist, then the difference of config[removeElements] and config[replaceWithChildrenElements] is empty.
  if (config.removeElements && config.replaceWithChildrenElements) {
    for (let removeElement of config.removeElements) {
      assert_false(
        config.replaceWithChildrenElements.some((replaceElement) =>
          is_same_sanitizer_name(removeElement, replaceElement),
        ),
        `replaceWithChildrenElements should not contain ${removeElement.name}`,
      );
    }
  }

  // If config[attributes] exists:
  if (config.attributes) {
  } else {
    if (config.elements) {
      for (let element of config.elements) {
        // Not both element[attributes] and element[removeAttributes] exist.
        assert_false("attributes" in element && "removeAttributes" in element,
                     `Element ${element.name} can't have both 'attributes' and 'removeAttributes'`);
      }
    }

    // config[dataAttributes] does not exist.
    assert_false("dataAttributes" in config, "dataAttributes does not exist");
  }
}

function assert_config(config, expected) {
  const PROPERTIES = [
    "attributes",
    "removeAttributes",
    "elements",
    "removeElements",
    "replaceWithChildrenElements",
    "comments",
    "dataAttributes",
  ];

  // Prevent some typos in the expected config.
  for (let key of Object.keys(expected)) {
    assert_in_array(key, PROPERTIES, "expected");
  }
  for (let key of Object.keys(config)) {
    assert_in_array(key, PROPERTIES, "config");
  }

  assert_config_is_valid(config);

  // XXX duplications
  // XXX other consistency checks

  function assert_attrs(key, config, expected, prefix = "config") {
    // XXX we allow specifying only a subset for expected.
    if (!(key in expected)) {
      return;
    }

    if (expected[key] === undefined) {
      assert_false(key in config, `Unexpected '${key}' in ${prefix}`);
      return;
    }

    assert_true(key in config, `Missing '${key}' from ${prefix}`);
    assert_equals(config[key]?.length, expected[key].length, `${prefix}.${key}.length`);
    for (let i = 0; i < expected[key].length; i++) {
      let attribute = expected[key][i];
      if (typeof attribute === "string") {
        assert_object_equals(
          config[key][i],
          { name: attribute, namespace: null },
          `${prefix}.${key}[${i}] should match`,
        );
      } else {
        assert_object_equals(
          config[key][i],
          attribute,
          `${prefix}.${key}[${i}] should match`,
        );
      }
    }
  }

  assert_attrs("attributes", config, expected);
  assert_attrs("removeAttributes", config, expected);

  function assert_elems(key) {
    if (!(key in expected)) {
      return;
    }

    if (expected[key] === undefined) {
      assert_false(key in config, `Unexpected '${key}' in config`);
      return;
    }

    assert_true(key in config, `Missing '${key}' from config`);
    assert_equals(config[key]?.length, expected[key].length, `${key}.length`);

    const XHTML_NS = "http://www.w3.org/1999/xhtml";

    for (let i = 0; i < expected[key].length; i++) {
      let element = expected[key][i];
      // To make writing tests a bit easier we also support the shorthand string syntax.
      if (typeof element === "string") {
        let extra = key === "elements" ? { removeAttributes: [] } : { };
        assert_object_equals(
          config[key][i],
          { name: element, namespace: XHTML_NS, ...extra },
          `${key}[${i}] should match`,
        );
      } else {
        if (key === "elements") {
          assert_equals(config[key][i].name, element.name, `${key}[${i}].name should match`);
          let ns = "namespace" in element ? element.namespace : XHTML_NS;
          assert_equals(config[key][i].namespace, ns, `${key}[${i}].namespace should match`);

          assert_attrs("attributes", config[key][i], element, `config.elements[${i}]`);
          assert_attrs("removeAttributes", config[key][i], element, `config.elements[${i}]`);
        } else {
          assert_object_equals(config[key][i], element, `${key}[${i}] should match`);
        }
      }
    }
  }

  assert_elems("elements");
  assert_elems("removeElements");
  assert_elems("replaceWithChildrenElements");

  if ("comments" in expected) {
    assert_equals(config.comments, expected.comments, "comments should match");
  }

  if ("dataAttributes" in expected) {
    assert_equals(config.dataAttributes, expected.dataAttributes, "dataAttributes should match");
  }
}
