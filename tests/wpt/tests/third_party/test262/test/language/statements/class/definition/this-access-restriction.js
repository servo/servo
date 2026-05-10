// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class this access restriction
---*/
class Base {}
(function() {
  class C extends Base {
    constructor() {
      var y;
      super();
    }
  }; new C();
}());
assert.throws(ReferenceError, function() {
  class C extends Base {
    constructor() {
      super(this.x);
    }
  }; new C();
});
assert.throws(ReferenceError, function() {
  class C extends Base {
    constructor() {
      super(this);
    }
  }; new C();
});
assert.throws(ReferenceError, function() {
  class C extends Base {
    constructor() {
      super.method();
      super(this);
    }
  }; new C();
});
assert.throws(ReferenceError, function() {
  class C extends Base {
    constructor() {
      super(super.method());
    }
  }; new C();
});
assert.throws(ReferenceError, function() {
  class C extends Base {
    constructor() {
      super(super());
    }
  }; new C();
});
assert.throws(ReferenceError, function() {
  class C extends Base {
    constructor() {
      super(1, 2, Object.getPrototypeOf(this));
    }
  }; new C();
});
(function() {
  class C extends Base {
    constructor() {
      { super(1, 2); }
    }
  }; new C();
}());
(function() {
  class C extends Base {
    constructor() {
      if (1) super();
    }
  }; new C();
}());

class C1 extends Object {
  constructor() {
    'use strict';
    super();
  }
};
new C1();

class C2 extends Object {
  constructor() {
    ; 'use strict';;;;;
    super();
  }
};
new C2();

class C3 extends Object {
  constructor() {
    ; 'use strict';;;;;
    // This is a comment.
    super();
  }
};
new C3();
