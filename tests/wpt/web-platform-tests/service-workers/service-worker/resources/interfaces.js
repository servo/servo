function verify_interface(name, instance, attributes) {
  assert_true(name in self,
              name + ' should be an defined type');
  if (instance) {
    assert_true(instance instanceof self[name],
                instance + ' should be an instance of ' + name);
    Object.keys(attributes || {}).forEach(function(attribute) {
        var type = attributes[attribute];
        assert_true(attribute in instance,
                    attribute + ' should be an attribute of ' + name);
        assert_equals(typeof instance[attribute], type,
                      attribute + ' should be of type ' + type);
      });
  }
}
