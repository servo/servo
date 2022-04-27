
function test_toggle_root_computed_values(property) {
  test_computed_value(property, 'none');
  test_computed_value(property, 'sticky sticky');
  test_computed_value(property, 'group group');
  test_computed_value(property, 'self self');
  test_computed_value(property, 'mytoggle');
  test_computed_value(property, 'mytoggle, yourtoggle');
  test_computed_value(property, 'mytoggle, mytoggle');
  test_computed_value(property, 'mytoggle 0 / 3 sticky self, yourtoggle 1 group self', 'mytoggle 3 sticky self, yourtoggle group self');
  test_computed_value(property, 'mytoggle 1 / 3 sticky self, yourtoggle 2 group self');
  test_computed_value(property, 'mytoggle 0/1', 'mytoggle');
  test_computed_value(property, 'mytoggle +0/1', 'mytoggle');
  test_computed_value(property, 'mytoggle 0/+1', 'mytoggle');
  test_computed_value(property, 'mytoggle -0/1', 'mytoggle');
  test_computed_value(property, 'mytoggle 2/+1', 'mytoggle 2 / 1');
  test_computed_value(property, 'mytoggle calc(-3)/1', 'mytoggle');
  test_computed_value(property, 'mytoggle 0/calc(-3)', 'mytoggle');
  test_computed_value(property, 'mytoggle calc(-3)/7', 'mytoggle 7');
  test_computed_value(property, 'mytoggle 7/calc(-3)', 'mytoggle 7 / 1');
  test_computed_value(property, 'mytoggle calc(6)/calc(9)', 'mytoggle 6 / 9');
  test_computed_value(property, 'mytoggle calc(6.4)/calc(9.6)', 'mytoggle 6 / 10');
  test_computed_value(property, 'mytoggle calc(6.5)/calc(-9.5)', 'mytoggle 7 / 1');
  test_computed_value(property, 'mytoggle group sticky self, yourtoggle self sticky', 'mytoggle sticky group self, yourtoggle sticky self');
  test_computed_value(property, 'mytoggle group 1 / 2', 'mytoggle 1 / 2 group');
}

function test_toggle_root_valid_values(property) {
  test_valid_value(property, 'none');
  test_valid_value(property, 'sticky sticky');
  test_valid_value(property, 'group group');
  test_valid_value(property, 'self self');
  test_valid_value(property, 'mytoggle');
  test_valid_value(property, 'mytoggle, yourtoggle');
  test_valid_value(property, 'mytoggle, mytoggle');
  test_valid_value(property, 'mytoggle 0 / 3 sticky self, yourtoggle 1 group self');
  test_valid_value(property, 'mytoggle 0/1', 'mytoggle 0 / 1');
  test_valid_value(property, 'mytoggle +0/1', 'mytoggle 0 / 1');
  test_valid_value(property, 'mytoggle 0/+1', 'mytoggle 0 / 1');
  test_valid_value(property, 'mytoggle -0/1', 'mytoggle 0 / 1');
  test_valid_value(property, 'mytoggle calc(-3) / 1');
  test_valid_value(property, 'mytoggle 0 / calc(-3)');
  test_valid_value(property, 'mytoggle calc(-3) / 7');
  test_valid_value(property, 'mytoggle 7 / calc(-3)');
  test_valid_value(property, 'mytoggle calc(6) / calc(9)');
  test_valid_value(property, 'mytoggle calc(6.4) / calc(9.6)');
  test_valid_value(property, 'mytoggle calc(6.5) / calc(-9.5)');
  test_valid_value(property, 'mytoggle group sticky self, yourtoggle self sticky', 'mytoggle sticky group self, yourtoggle sticky self');
  test_valid_value(property, 'mytoggle group 1 / 2', 'mytoggle 1 / 2 group');
}

function test_toggle_root_invalid_values(property) {
  test_invalid_value(property, 'none 1');
  test_invalid_value(property, 'none sticky');
  test_invalid_value(property, 'none group');
  test_invalid_value(property, 'none self');
  test_invalid_value(property, 'None self');
  test_invalid_value(property, 'NONE self');
  test_invalid_value(property, 'mytoggle sticky sticky');
  test_invalid_value(property, 'mytoggle group group');
  test_invalid_value(property, 'mytoggle self self');
  test_invalid_value(property, 'none sticky sticky');
  test_invalid_value(property, 'none group group');
  test_invalid_value(property, 'none self self');
  test_invalid_value(property, 'none, mytoggle');
  test_invalid_value(property, 'mytoggle, none');
  test_invalid_value(property, 'mytoggle 0 sticky self');
  test_invalid_value(property, 'mytoggle 0 / 0 sticky self');
  test_invalid_value(property, 'mytoggle 1 / -1 sticky self');
  test_invalid_value(property, 'mytoggle -1 / 1 sticky self');
  test_invalid_value(property, 'mytoggle -1 / -1 sticky self');
  test_invalid_value(property, 'mytoggle 0/-1');
  test_invalid_value(property, 'mytoggle 0/0');
  test_invalid_value(property, 'mytoggle 0/-0');
  test_invalid_value(property, 'mytoggle 0/+0');
  test_invalid_value(property, 'mytoggle sticky 1 / 3 group self sticky');
  test_invalid_value(property, 'mytoggle sticky 1 / 3 group self group');
  test_invalid_value(property, 'mytoggle sticky 1 / 3 group self self');
  test_invalid_value(property, 'mytoggle sticky 1 / 3 group self 1');
  test_invalid_value(property, 'mytoggle sticky 1 / group');
  test_invalid_value(property, 'mytoggle sticky 1 / group 1');
}
