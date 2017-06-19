// https://drafts.fxtf.org/geometry/#DOMMatrix

importScripts("/resources/testharness.js");

['DOMMatrix', 'DOMMatrixReadOnly'].forEach(constr => {
  test(() => {
    assert_true(constr in self, `${constr} should exist`);
    assert_throws(new TypeError(), () => new self[constr]('matrix(1,0,0,1,0,0)') );
  }, `${constr} constructor with string argument in worker`);

  test(() => {
    assert_true(constr in self, `${constr} should exist`);
    assert_throws(new TypeError(), () => new self[constr]('') );
  }, `${constr} constructor with empty string argument in worker`);

  test(() => {
    const matrix = new self[constr]();
    assert_equals(String(matrix), `[object ${constr}]`);
  }, `${constr} stringifier in worker (2d identity)`);

  test(() => {
    const matrix = self[constr].fromMatrix({is2D: false});
    assert_equals(String(matrix), `[object ${constr}]`);
  }, `${constr} stringifier in worker (3d identity)`);

  test(() => {
    const matrix = new self[constr]([1, 0, 0, NaN, Infinity, -Infinity]);
    assert_equals(String(matrix), `[object ${constr}]`);
  }, `${constr} stringifier in worker (non-finite values)`);
});

test(() => {
  assert_false('setMatrixValue' in DOMMatrix.prototype, 'on prototype');
  const matrix = new DOMMatrix();
  assert_false('setMatrixValue' in matrix, 'on instance');
}, 'DOMMatrix setMatrixValue in worker');

done();
