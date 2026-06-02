// META: global=window,worker

'use strict';

test(() => {
  const error = new WebSocketError();
  assert_equals(error.code, 0, 'DOMException code should be 0');
  assert_equals(error.name, 'WebSocketError', 'name should be correct');
  assert_equals(error.message, '', 'DOMException message should be empty');
  assert_equals(error.closeCode, null, 'closeCode should be null');
  assert_equals(error.reason, '', 'reason should be empty');
}, 'WebSocketError defaults should be correct');

test(() => {
  const error = new WebSocketError('message', { closeCode: 3456, reason: 'reason' });
  assert_equals(error.code, 0, 'DOMException code should be 0');
  assert_equals(error.name, 'WebSocketError', 'name should be correct');
  assert_equals(error.message, 'message', 'DOMException message should be set');
  assert_equals(error.closeCode, 3456, 'closeCode should match');
  assert_equals(error.reason, 'reason', 'reason should match');
}, 'WebSocketError should be initialised from arguments');

for (const invalidCode of [999, 1001, 2999, 5000]) {
  test(() => {
    assert_throws_dom('InvalidAccessError', () => new WebSocketError('', { closeCode: invalidCode }),
                      'invalid code should throw an InvalidAccessError');
  }, `new WebSocketError with invalid code ${invalidCode} should throw`);
}

test(() => {
  const error = new WebSocketError('', { closeCode: 3333 });
  assert_equals(error.closeCode, 3333, 'code should be used');
  assert_equals(error.reason, '', 'reason should be empty');
}, 'passing only close code to WebSocketError should work');

test(() => {
  const error = new WebSocketError('', { reason: 'specified' });
  assert_equals(error.closeCode, 1000, 'code should be defaulted');
  assert_equals(error.reason, 'specified', 'reason should be used');
}, 'passing a non-empty reason should cause the close code to be set to 1000');

test(() => {
  assert_throws_dom('SyntaxError', () => new WebSocketError('', { closeCode: 1000, reason: 'x'.repeat(124) }),
                    'overlong reason should trigger SyntaxError');
}, 'overlong reason should throw');

test(() => {
  assert_throws_dom('SyntaxError', () => new WebSocketError('', { reason: '🔌'.repeat(32) }),
                    'overlong reason should throw');
}, 'reason should be rejected based on utf-8 bytes, not character count');
