var validator = {
  test_tooLong: function(ctl, data) {
    var self = this;
    test(function () {
       self.pre_check(ctl, "tooLong");
      self.set_conditions(ctl, data.conditions);
      if (data.dirty)
        self.set_dirty(ctl);

      if (data.expected)
        assert_true(ctl.validity.tooLong, "The validity.tooLong should be true.");
      else
        assert_false(ctl.validity.tooLong, "The validity.tooLong should be false.");
    }, data.name);
  },

  test_tooShort: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "tooShort");
      self.set_conditions(ctl, data.conditions);
      if (data.dirty)
        self.set_dirty(ctl);

      if (data.expected)
        assert_true(ctl.validity.tooShort, "The validity.tooShort should be true.");
      else
        assert_false(ctl.validity.tooShort, "The validity.tooShort should be false.");
    }, data.name);
  },

  test_patternMismatch: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "patternMismatch");
      self.set_conditions(ctl, data.conditions);

      if (data.expected)
        assert_true(ctl.validity.patternMismatch, "The validity.patternMismatch should be true.");
      else
        assert_false(ctl.validity.patternMismatch, "The validity.patternMismatch should be false.");
    }, data.name);
  },

  test_valueMissing: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "valueMissing");
      self.set_conditions(ctl, data.conditions);
      if (data.expected)
        assert_true(ctl.validity.valueMissing, "The validity.valueMissing should be true.");
      else
        assert_false(ctl.validity.valueMissing, "The validity.valueMissing should be false.");
    }, data.name);
  },

  test_typeMismatch: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "typeMismatch");
      self.set_conditions(ctl, data.conditions);

      if (data.expected)
        assert_true(ctl.validity.typeMismatch, "The validity.typeMismatch should be true.");
      else
        assert_false(ctl.validity.typeMismatch, "The validity.typeMismatch should be false.");
    }, data.name);
  },

  test_rangeOverflow: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "rangeOverflow");
      self.set_conditions(ctl, data.conditions);

      if (data.expected)
        assert_true(ctl.validity.rangeOverflow, "The validity.rangeOverflow should be true.");
      else
        assert_false(ctl.validity.rangeOverflow, "The validity.rangeOverflow should be false.");
    }, data.name);
  },

  test_rangeUnderflow: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "rangeUnderflow");
      self.set_conditions(ctl, data.conditions);
      if (data.expected)
        assert_true(ctl.validity.rangeUnderflow, "The validity.rangeUnderflow should be true.");
      else
        assert_false(ctl.validity.rangeUnderflow, "The validity.rangeUnderflow should be false.");
    }, data.name);
  },

  test_stepMismatch: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "stepMismatch");
      self.set_conditions(ctl, data.conditions);

      if (data.expected)
        assert_true(ctl.validity.stepMismatch, "The validity.stepMismatch should be true.");
      else
        assert_false(ctl.validity.stepMismatch, "The validity.stepMismatch should be false.");
    }, data.name);
  },

  test_badInput: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "badInput");
      self.set_conditions(ctl, data.conditions);

      if (data.expected)
        assert_true(ctl.validity.badInput, "The validity.badInput should be true.");
      else
        assert_false(ctl.validity.badInput, "The validity.badInput should be false.");
    }, data.name);
  },

  test_customError: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "customError");
      ctl.setCustomValidity(data.conditions.message);

      if (data.expected) {
        assert_true(ctl.validity.customError, "The validity.customError attribute should be true.");
        assert_equals(ctl.validationMessage, data.conditions.message,
                      "The validationMessage attribute should be '" + data.conditions.message + "'.");
      } else {
        assert_false(ctl.validity.customError, "The validity.customError attribute should be false.");
        assert_equals(ctl.validationMessage, "", "The validationMessage attribute must be empty.");
      }
    }, data.name);
  },

  test_isValid: function (ctl, data) {
    var self = this;
    test(function () {
      self.set_conditions(ctl, data.conditions);
      if (data.dirty)
        self.set_dirty(ctl);

      if (data.expected)
        assert_true(ctl.validity.valid, "The validity.valid should be true.");
      else
        assert_false(ctl.validity.valid, "The validity.valid should be false.");
    }, data.name);
  },

  test_willValidate: function(ctl, data) {
    var self = this;
    test(function () {
      self.pre_check(ctl, "willValidate");
      self.set_conditions(ctl, data.conditions);
      if (data.ancestor) {
        var dl = document.createElement("datalist");
        dl.appendChild(ctl);
      }

      if (data.expected)
        assert_true(ctl.willValidate, "The willValidate attribute should be true.");
      else
        assert_false(ctl.willValidate, "The willValidate attribute should be false.");
    }, data.name);
  },

  test_checkValidity: function (ctl, data) {
    var self = this;
    test(function () {
      var eventFired = false;
      self.pre_check(ctl, "checkValidity");
      self.set_conditions(ctl, data.conditions);
      if (data.dirty)
        self.set_dirty(ctl);

      on_event(ctl, "invalid", function(e){
        assert_equals(e.type, "invalid", "The invalid event should be fired.");
        eventFired = true;
      });

      if (data.expected) {
        assert_true(ctl.checkValidity(), "The checkValidity method should be true.");
        assert_false(eventFired, "The invalid event should not be fired.");
      } else {
        assert_false(ctl.checkValidity(), "The checkValidity method should be false.");
        assert_true(eventFired, "The invalid event should be fired.");
      }
    }, data.name);

    test(function () {
      var fm = document.createElement("form");
      var ctl2 = ctl.cloneNode(true);

      self.pre_check(ctl, "checkValidity");
      self.set_conditions(ctl2, data.conditions);
      fm.appendChild(ctl2);
      document.body.appendChild(fm);
      if (data.dirty)
        self.set_dirty(ctl2);

      var result = fm.checkValidity();
      document.body.removeChild(fm);

      if (data.expected)
        assert_true(result, "The checkValidity method of the element's form owner should return true.");
      else
        assert_false(result, "The checkValidity method of the element's form owner should return false.");
    }, data.name + " (in a form)");
  },

  test_reportValidity: function (ctl, data) {
    var self = this;
    test(function () {
      var eventFired = false;

      self.pre_check(ctl, "reportValidity");
      self.set_conditions(ctl, data.conditions);
      if (data.dirty)
        self.set_dirty(ctl);

      on_event(ctl, "invalid", function(e){
        assert_equals(e.type, "invalid", "The invalid event should be fired.");
        eventFired = true;
      });

      if (data.expected) {
        assert_true(ctl.reportValidity(), "The reportValidity method should be true.");
        assert_false(eventFired, "The invalid event should not be fired.");
      } else {
        assert_false(ctl.reportValidity(), "The reportValidity method should be false.");
        assert_true(eventFired, "The invalid event should be fired.");
      }
    }, data.name);

    test(function () {
      var fm = document.createElement("form");
      var ctl2 = ctl.cloneNode(true);

      self.pre_check(ctl, "reportValidity");
      self.set_conditions(ctl2, data.conditions);
      fm.appendChild(ctl2);
      document.body.appendChild(fm);
      if (data.dirty)
        self.set_dirty(ctl2);

      var result = fm.reportValidity();
      document.body.removeChild(fm);

      if (data.expected)
        assert_true(result, "The reportValidity method of the element's form owner should return true.");
      else
        assert_false(result, "The reportValidity method of the element's form owner should return false.");
    }, data.name + " (in a form)");
  },

  test_support_type: function (ctl, typ, testName) {
    test(function () {
      assert_equals(ctl.type, typ, "The " + typ + " type should be supported.");
    }, testName);
  },

  set_conditions: function (ctl, obj) {
    [
      "checked",
      "disabled",
      "max",
      "maxlength",
      "min",
      "minlength",
      "multiple",
      "pattern",
      "required",
      "selected",
      "step",
      "value"
    ].forEach(function(item) {
      ctl.removeAttribute(item);
    });
    for (var attr in obj) {
      if (attr === "checked" || obj[attr] || obj[attr] === "")
        ctl[attr] = obj[attr];
    }
  },

  set_dirty: function(ctl) {
    ctl.focus();
    var old_value = ctl.value;
    ctl.value = "a";
    ctl.value = old_value;
  },

  pre_check: function(ctl, item) {
    switch (item) {
      case "willValidate":
        assert_true(item in ctl, "The " + item + " attribute doesn't exist.");
        break;
      case "checkValidity":
      case "reportValidity":
        assert_true(item in ctl, "The " + item + " method doesn't exist.");
        break;
      case "tooLong":
      case "tooShort":
      case "patternMismatch":
      case "typeMismatch":
      case "stepMismatch":
      case "rangeOverflow":
      case "rangeUnderflow":
      case "valueMissing":
      case "badInput":
      case "valid":
        assert_true("validity" in ctl, "The validity attribute doesn't exist.");
        assert_true(item in ctl.validity, "The " + item + " attribute doesn't exist.");
        break;
      case "customError":
        assert_true("validity" in ctl, "The validity attribute doesn't exist.");
        assert_true("setCustomValidity" in ctl, "The validity attribute doesn't exist.");
        assert_true("validationMessage" in ctl, "The validity attribute doesn't exist.");
        assert_true(item in ctl.validity, "The " + item + " attribute doesn't exist.");
        break;
    }
  },

  run_test: function (testee, method) {
    var testMethod = "test_" + method;
    if (typeof this[testMethod] !== "function") {
      return false;
    }

    var ele = null,
        prefix = "";

    for (var i = 0; i < testee.length; i++) {
      if (testee[i].types.length > 0) {
        for (var typ in testee[i].types) {
          ele = document.createElement(testee[i].tag);
          document.body.appendChild(ele);
          try {
            ele.type = testee[i].types[typ];
          } catch (e) {
            //Do nothing, avoid the runtime error breaking the test
          }

          prefix = "[" + testee[i].tag.toUpperCase() + " in " + testee[i].types[typ].toUpperCase() + " status] ";
          if (ele.type != testee[i].types[typ]) {
            this.test_support_type(
              ele,
              testee[i].types[typ],
              prefix + "The " + testee[i].types[typ] + " type must be supported."
            );
            continue;
          }

          for (var j = 0; j < testee[i].testData.length; j++) {
            testee[i].testData[j].name = testee[i].testData[j].name.replace(/\[.*\]\s/g, prefix);
            this[testMethod](ele, testee[i].testData[j]);
          }
        }
      } else {
        ele = document.createElement(testee[i].tag);
        document.body.appendChild(ele);
        prefix = "[" + testee[i].tag + "] ";

        if (testElements[i].tag === "select") {
          ele.add(new Option("test1", ""));
          ele.add(new Option("test2", 1));
        }

        for (var item in testee[i].testData) {
          testee[i].testData[item].name = testee[i].testData[item].name.replace("[target]", prefix);
          this[testMethod](ele, testee[i].testData[item]);
        }
      }
    }
  }
}
