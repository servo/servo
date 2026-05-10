// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: unsigned-right-shift operator ToNumeric with BigInt operands
esid: sec-unsigned-right-shift-operator-runtime-semantics-evaluation
info: After ToNumeric type coercion, unsigned-right-shift always throws for BigInt operands
features: [BigInt, Symbol.toPrimitive, computed-property-names]
---*/
function err() {
  throw new Test262Error();
}

function MyError() {}

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: function() {
      return 2n;
    },

    valueOf: err,
    toString: err
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: function() {return 2n;}, valueOf: err, toString: err}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: function() {
      return 2n;
    },

    valueOf: err,
    toString: err
  };
}, '0n >>> {[Symbol.toPrimitive]: function() {return 2n;}, valueOf: err, toString: err} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: function() {
      return 2n;
    },

    toString: err
  }) >>> 0n;
}, '({valueOf: function() {return 2n;}, toString: err}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: function() {
      return 2n;
    },

    toString: err
  };
}, '0n >>> {valueOf: function() {return 2n;}, toString: err} throws TypeError');

assert.throws(TypeError, function() {
  ({
    toString: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({toString: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    toString: function() {
      return 2n;
    }
  };
}, '0n >>> {toString: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: undefined,

    valueOf: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: undefined, valueOf: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: undefined,

    valueOf: function() {
      return 2n;
    }
  };
}, '0n >>> {[Symbol.toPrimitive]: undefined, valueOf: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: null,

    valueOf: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: null, valueOf: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: null,

    valueOf: function() {
      return 2n;
    }
  };
}, '0n >>> {[Symbol.toPrimitive]: null, valueOf: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: null,

    toString: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({valueOf: null, toString: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: null,

    toString: function() {
      return 2n;
    }
  };
}, '0n >>> {valueOf: null, toString: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: 1,

    toString: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({valueOf: 1, toString: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: 1,

    toString: function() {
      return 2n;
    }
  };
}, '0n >>> {valueOf: 1, toString: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: {},

    toString: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({valueOf: {}, toString: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: {},

    toString: function() {
      return 2n;
    }
  };
}, '0n >>> {valueOf: {}, toString: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: function() {
      return {};
    },

    toString: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({valueOf: function() {return {};}, toString: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: function() {
      return {};
    },

    toString: function() {
      return 2n;
    }
  };
}, '0n >>> {valueOf: function() {return {};}, toString: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: function() {
      return Object(12345);
    },

    toString: function() {
      return 2n;
    }
  }) >>> 0n;
}, '({valueOf: function() {return Object(12345);}, toString: function() {return 2n;}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: function() {
      return Object(12345);
    },

    toString: function() {
      return 2n;
    }
  };
}, '0n >>> {valueOf: function() {return Object(12345);}, toString: function() {return 2n;}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: 1
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: 1}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: 1
  };
}, '0n >>> {[Symbol.toPrimitive]: 1} throws TypeError');

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: {}
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: {}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: {}
  };
}, '0n >>> {[Symbol.toPrimitive]: {}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: function() {return Object(1);}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  };
}, '0n >>> {[Symbol.toPrimitive]: function() {return Object(1);}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    [Symbol.toPrimitive]: function() {
      return {};
    }
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: function() {return {};}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    [Symbol.toPrimitive]: function() {
      return {};
    }
  };
}, '0n >>> {[Symbol.toPrimitive]: function() {return {};}} throws TypeError');

assert.throws(MyError, function() {
  ({
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  }) >>> 0n;
}, '({[Symbol.toPrimitive]: function() {throw new MyError();}}) >>> 0n throws MyError');

assert.throws(MyError, function() {
  0n >>> {
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  };
}, '0n >>> {[Symbol.toPrimitive]: function() {throw new MyError();}} throws MyError');

assert.throws(MyError, function() {
  ({
    valueOf: function() {
      throw new MyError();
    }
  }) >>> 0n;
}, '({valueOf: function() {throw new MyError();}}) >>> 0n throws MyError');

assert.throws(MyError, function() {
  0n >>> {
    valueOf: function() {
      throw new MyError();
    }
  };
}, '0n >>> {valueOf: function() {throw new MyError();}} throws MyError');

assert.throws(MyError, function() {
  ({
    toString: function() {
      throw new MyError();
    }
  }) >>> 0n;
}, '({toString: function() {throw new MyError();}}) >>> 0n throws MyError');

assert.throws(MyError, function() {
  0n >>> {
    toString: function() {
      throw new MyError();
    }
  };
}, '0n >>> {toString: function() {throw new MyError();}} throws MyError');

assert.throws(TypeError, function() {
  ({
    valueOf: null,
    toString: null
  }) >>> 0n;
}, '({valueOf: null, toString: null}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: null,
    toString: null
  };
}, '0n >>> {valueOf: null, toString: null} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: 1,
    toString: 1
  }) >>> 0n;
}, '({valueOf: 1, toString: 1}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: 1,
    toString: 1
  };
}, '0n >>> {valueOf: 1, toString: 1} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: {},
    toString: {}
  }) >>> 0n;
}, '({valueOf: {}, toString: {}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: {},
    toString: {}
  };
}, '0n >>> {valueOf: {}, toString: {}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: function() {
      return Object(1);
    },

    toString: function() {
      return Object(1);
    }
  }) >>> 0n;
}, '({valueOf: function() {return Object(1);}, toString: function() {return Object(1);}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: function() {
      return Object(1);
    },

    toString: function() {
      return Object(1);
    }
  };
}, '0n >>> {valueOf: function() {return Object(1);}, toString: function() {return Object(1);}} throws TypeError');

assert.throws(TypeError, function() {
  ({
    valueOf: function() {
      return {};
    },

    toString: function() {
      return {};
    }
  }) >>> 0n;
}, '({valueOf: function() {return {};}, toString: function() {return {};}}) >>> 0n throws TypeError');

assert.throws(TypeError, function() {
  0n >>> {
    valueOf: function() {
      return {};
    },

    toString: function() {
      return {};
    }
  };
}, '0n >>> {valueOf: function() {return {};}, toString: function() {return {};}} throws TypeError');
