(function() {

  setup("", {explicit_done: true});

  /**
  *
  * partial interface Navigator {
  *     Promise<BatteryManager> getBattery ();
  * };
  *
  */

  test(function() {
    assert_idl_attribute(navigator, 'getBattery', 'navigator must have getBattery attribute');
  }, 'getBattery is present on navigator');

  navigator.getBattery().then(function (battery) {

    /**
    *
    * interface BatteryManager : EventTarget {
    *     readonly attribute boolean             charging;
    *     readonly attribute unrestricted double chargingTime;
    *     readonly attribute unrestricted double dischargingTime;
    *     readonly attribute double              level;
    *              attribute EventHandler        onchargingchange;
    *              attribute EventHandler        onchargingtimechange;
    *              attribute EventHandler        ondischargingtimechange;
    *              attribute EventHandler        onlevelchange;
    * };
    *
    */

    // interface BatteryManager : EventTarget {

    test(function() {
      assert_own_property(window, 'BatteryManager');
    }, 'window has an own property BatteryManager');

    test(function() {
      assert_true(battery instanceof EventTarget);
    }, 'battery inherits from EventTarget');

    // readonly attribute boolean charging;

    test(function() {
      assert_idl_attribute(battery, 'charging', 'battery must have charging attribute');
    }, 'charging attribute exists');

    test(function() {
      assert_readonly(battery, 'charging', 'charging must be readonly')
    }, 'charging attribute is readonly');

    // readonly attribute unrestricted double chargingTime;

    test(function() {
      assert_idl_attribute(battery, 'chargingTime', 'battery must have chargingTime attribute');
    }, 'chargingTime attribute exists');

    test(function() {
      assert_readonly(battery, 'chargingTime', 'chargingTime must be readonly')
    }, 'chargingTime attribute is readonly');

    // readonly attribute unrestricted double dischargingTime;

    test(function() {
      assert_idl_attribute(battery, 'dischargingTime', 'battery must have dischargingTime attribute');
    }, 'dischargingTime attribute exists');

    test(function() {
      assert_readonly(battery, 'dischargingTime', 'dischargingTime must be readonly')
    }, 'dischargingTime attribute is readonly');

    // readonly attribute double level;

    test(function() {
      assert_idl_attribute(battery, 'level', 'battery must have level attribute');
    }, 'level attribute exists');

    test(function() {
      assert_readonly(battery, 'level', 'level must be readonly')
    }, 'level attribute is readonly');

    // attribute EventHandler onchargingchange;

    test(function() {
      assert_idl_attribute(battery, 'onchargingchange', 'battery must have onchargingchange attribute');
    }, 'onchargingchange attribute exists');

    test(function() {
      assert_equals(battery.onchargingchange, null, 'onchargingchange must be null')
    }, 'onchargingchange is null');

    test(function() {
      var desc = 'onchargingchange did not accept callable object',
          func = function() {},
          desc = 'Expected to find onchargingchange attribute on battery object';
      assert_idl_attribute(battery, 'onchargingchange', desc);
      window.onchargingchange = func;
      assert_equals(window.onchargingchange, func, desc);
    }, 'onchargingchange is set to function');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = {};
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat object as null');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = {
          call: 'test'
      };
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat object with non-callable call property as null');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable (string) as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = 'string';
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat string as null');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable (number) as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = 123;
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat number as null');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable (undefined) as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = undefined;
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat undefined as null');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable (array) as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = [];
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat array as null');

    test(function() {
      var desc = 'onchargingchange did not treat noncallable host object as null';
      battery.onchargingchange = function() {};
      battery.onchargingchange = Node;
      assert_equals(battery.onchargingchange, null, desc);
    }, 'onchargingchange: treat non-callable host object as null');

    // attribute EventHandler onchargingtimechange;

    test(function() {
      assert_idl_attribute(battery, 'onchargingtimechange', 'battery must have onchargingtimechange attribute');
    }, 'onchargingtimechange attribute exists');

    test(function() {
      assert_equals(battery.onchargingtimechange, null, 'onchargingtimechange must be null')
    }, 'onchargingtimechange is null');

    test(function() {
      var desc = 'onchargingtimechange did not accept callable object',
          func = function() {},
          desc = 'Expected to find onchargingtimechange attribute on battery object';
      assert_idl_attribute(battery, 'onchargingtimechange', desc);
      window.onchargingtimechange = func;
      assert_equals(window.onchargingtimechange, func, desc);
    }, 'onchargingtimechange is set to function');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = {};
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat object as null');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = {
          call: 'test'
      };
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat object with non-callable call property as null');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable (string) as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = 'string';
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat string as null');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable (number) as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = 123;
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat number as null');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable (undefined) as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = undefined;
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat undefined as null');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable (array) as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = [];
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat array as null');

    test(function() {
      var desc = 'onchargingtimechange did not treat noncallable host object as null';
      battery.onchargingtimechange = function() {};
      battery.onchargingtimechange = Node;
      assert_equals(battery.onchargingtimechange, null, desc);
    }, 'onchargingtimechange: treat non-callable host object as null');

    // attribute EventHandler ondischargingtimechange;

    test(function() {
      assert_idl_attribute(battery, 'ondischargingtimechange', 'battery must have ondischargingtimechange attribute');
    }, 'ondischargingtimechange attribute exists');

    test(function() {
      assert_equals(battery.ondischargingtimechange, null, 'ondischargingtimechange must be null')
    }, 'ondischargingtimechange is null');

    test(function() {
      var desc = 'ondischargingtimechange did not accept callable object',
          func = function() {},
          desc = 'Expected to find ondischargingtimechange attribute on battery object';
      assert_idl_attribute(battery, 'ondischargingtimechange', desc);
      window.ondischargingtimechange = func;
      assert_equals(window.ondischargingtimechange, func, desc);
    }, 'ondischargingtimechange is set to function');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = {};
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat object as null');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = {
          call: 'test'
      };
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat object with non-callable call property as null');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable (string) as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = 'string';
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat string as null');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable (number) as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = 123;
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat number as null');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable (undefined) as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = undefined;
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat undefined as null');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable (array) as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = [];
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat array as null');

    test(function() {
      var desc = 'ondischargingtimechange did not treat noncallable host object as null';
      battery.ondischargingtimechange = function() {};
      battery.ondischargingtimechange = Node;
      assert_equals(battery.ondischargingtimechange, null, desc);
    }, 'ondischargingtimechange: treat non-callable host object as null');

    // attribute EventHandler onlevelchange;

    test(function() {
      assert_idl_attribute(battery, 'onlevelchange', 'battery must have onlevelchange attribute');
    }, 'onlevelchange attribute exists');

    test(function() {
      assert_equals(battery.onlevelchange, null, 'onlevelchange must be null')
    }, 'onlevelchange is null');

    test(function() {
      var desc = 'onlevelchange did not accept callable object',
          func = function() {},
          desc = 'Expected to find onlevelchange attribute on battery object';
      assert_idl_attribute(battery, 'onlevelchange', desc);
      window.onlevelchange = func;
      assert_equals(window.onlevelchange, func, desc);
    }, 'onlevelchange is set to function');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = {};
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat object as null');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = {
          call: 'test'
      };
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat object with non-callable call property as null');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable (string) as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = 'string';
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat string as null');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable (number) as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = 123;
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat number as null');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable (undefined) as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = undefined;
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat undefined as null');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable (array) as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = [];
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat array as null');

    test(function() {
      var desc = 'onlevelchange did not treat noncallable host object as null';
      battery.onlevelchange = function() {};
      battery.onlevelchange = Node;
      assert_equals(battery.onlevelchange, null, desc);
    }, 'onlevelchange: treat non-callable host object as null');

    done();

  }, function () {});

})();
