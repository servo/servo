/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from './util.js'; // The state of the preprocessor is a stack of States.
var
State = /*#__PURE__*/function (State) {State[State["Seeking"] = 0] = "Seeking";State[State["Passing"] = 1] = "Passing";State[State["Skipping"] = 2] = "Skipping";return State;}(State || {});


// Have already seen a passing condition; now skipping the rest


// The transitions in the state space are the following preprocessor directives:
// - Sibling elif
// - Sibling else
// - Sibling endif
// - Child if
class Directive {


  constructor(depth) {
    this.depth = depth;
  }

  checkDepth(stack) {
    assert(
      stack.length === this.depth,
      `Number of "$"s must match nesting depth, currently ${stack.length} (e.g. $if $$if $$endif $endif)`
    );
  }


}

class If extends Directive {


  constructor(depth, predicate) {
    super(depth);
    this.predicate = predicate;
  }

  applyTo(stack) {
    this.checkDepth(stack);
    const parentState = stack[stack.length - 1].state;
    stack.push({
      allowsFollowingElse: true,
      state:
      parentState !== State.Passing ?
      State.Skipping :
      this.predicate ?
      State.Passing :
      State.Seeking
    });
  }
}

class ElseIf extends If {
  applyTo(stack) {
    assert(stack.length >= 1);
    const { allowsFollowingElse, state: siblingState } = stack.pop();
    this.checkDepth(stack);
    assert(allowsFollowingElse, 'pp.elif after pp.else');
    if (siblingState !== State.Seeking) {
      stack.push({ allowsFollowingElse: true, state: State.Skipping });
    } else {
      super.applyTo(stack);
    }
  }
}

class Else extends Directive {
  applyTo(stack) {
    assert(stack.length >= 1);
    const { allowsFollowingElse, state: siblingState } = stack.pop();
    this.checkDepth(stack);
    assert(allowsFollowingElse, 'pp.else after pp.else');
    stack.push({
      allowsFollowingElse: false,
      state: siblingState === State.Seeking ? State.Passing : State.Skipping
    });
  }
}

class EndIf extends Directive {
  applyTo(stack) {
    stack.pop();
    this.checkDepth(stack);
  }
}

/**
 * A simple template-based, non-line-based preprocessor implementing if/elif/else/endif.
 *
 * @example
 * ```
 *     const shader = pp`
 * ${pp._if(expr)}
 *   const x: ${type} = ${value};
 * ${pp._elif(expr)}
 * ${pp.__if(expr)}
 * ...
 * ${pp.__else}
 * ...
 * ${pp.__endif}
 * ${pp._endif}`;
 * ```
 *
 * @param strings - The array of constant string chunks of the template string.
 * @param ...values - The array of interpolated `${}` values within the template string.
 */
export function pp(
strings,
...values)
{
  let result = '';
  const stateStack = [{ allowsFollowingElse: false, state: State.Passing }];

  for (let i = 0; i < values.length; ++i) {
    const passing = stateStack[stateStack.length - 1].state === State.Passing;
    if (passing) {
      result += strings[i];
    }

    const value = values[i];
    if (value instanceof Directive) {
      value.applyTo(stateStack);
    } else {
      if (passing) {
        result += value;
      }
    }
  }
  assert(stateStack.length === 1, 'Unterminated preprocessor condition at end of file');
  result += strings[values.length];

  return result;
}
pp._if = (predicate) => new If(1, predicate);
pp._elif = (predicate) => new ElseIf(1, predicate);
pp._else = new Else(1);
pp._endif = new EndIf(1);
pp.__if = (predicate) => new If(2, predicate);
pp.__elif = (predicate) => new ElseIf(2, predicate);
pp.__else = new Else(2);
pp.__endif = new EndIf(2);
pp.___if = (predicate) => new If(3, predicate);
pp.___elif = (predicate) => new ElseIf(3, predicate);
pp.___else = new Else(3);
pp.___endif = new EndIf(3);
// Add more if needed.