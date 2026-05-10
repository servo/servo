// Copyright (C) 2024 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-ConditionalExpression
description: >
    await binds more tightly than conditional operators
info: |
    ConditionalExpression[In, Yield, Await] :
      ShortCircuitExpression[?In, ?Yield, ?Await]
      ShortCircuitExpression[?In, ?Yield, ?Await] `?` AssignmentExpression[+In, ?Yield, ?Await] `:` AssignmentExpression[?In, ?Yield, ?Await]
    
    ShortCircuitExpression[In, Yield, Await] :
      LogicalORExpression[?In, ?Yield, ?Await]
      CoalesceExpression[?In, ?Yield, ?Await]
    
    LogicalORExpression[In, Yield, Await] :
      LogicalANDExpression[?In, ?Yield, ?Await]
      LogicalORExpression[?In, ?Yield, ?Await] `||` LogicalANDExpression[?In, ?Yield, ?Await]
    
    LogicalANDExpression[In, Yield, Await] :
      BitwiseORExpression[?In, ?Yield, ?Await]
      LogicalANDExpression[?In, ?Yield, ?Await] `&&` BitwiseORExpression[?In, ?Yield, ?Await]
    
    BitwiseORExpression[In, Yield, Await] :
      BitwiseXORExpression[?In, ?Yield, ?Await]
    
    BitwiseXORExpression[In, Yield, Await] :
      BitwiseANDExpression[?In, ?Yield, ?Await]
    
    BitwiseANDExpression[In, Yield, Await] :
      EqualityExpression[?In, ?Yield, ?Await]
    
    EqualityExpression[In, Yield, Await] :
      RelationalExpression[?In, ?Yield, ?Await]
    
    RelationalExpression[In, Yield, Await] :
      ShiftExpression[?Yield, ?Await]
    
    ShiftExpression[Yield, Await] :
      AdditiveExpression[?Yield, ?Await]
    
    AdditiveExpression[Yield, Await] :
      MultiplicativeExpression[?Yield, ?Await]
    
    MultiplicativeExpression[Yield, Await] :
      ExponentiationExpression[?Yield, ?Await]
    
    ExponentiationExpression[Yield, Await] :
      UnaryExpression[?Yield, ?Await]
    
    UnaryExpression[Yield, Await] :
      UpdateExpression[?Yield, ?Await]
      [+Await] AwaitExpression[?Yield]
    
    AwaitExpression[Yield] :
      `await` UnaryExpression[?Yield, +Await]
flags: [async]
includes: [asyncHelpers.js]
---*/

async function foo() {
  let x = 'initial value';
  let shouldNotBeAwaited = {
    then: function(onFulfilled) {
      x = 'unexpected then() call';
      Promise.resolve().then(onFulfilled);
    }
  };
  // This must parse like `(await false) || shouldNotBeAwaited`.
  await false || shouldNotBeAwaited;
  assert.sameValue(x, 'initial value');
}
asyncTest(foo);
