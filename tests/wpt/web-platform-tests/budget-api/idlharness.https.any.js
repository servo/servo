// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// See https://wicg.github.io/budget-api/

idl_test(
  ['budget-api'],
  ['html'],
  async idl_array => {
    idl_array.add_objects({ BudgetService: ['navigator.budget'] });
    if (self.Window) {
      idl_array.add_objects({ Navigator: ['navigator'] });
    } else {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    }
    const budgetStates = await navigator.budget.getBudget();
    if (budgetStates.length) {
      self.budgetState = budgetStates[0];
      idl_array.add_objects({ BudgetState: ['budgetState'] });
    }
  }
);
