// |matrix| - Float32Array, |input| - point-like dict (must have x, y, z, w)
let transform_point_by_matrix = function(matrix, input) {
  return {
    x : matrix[0] * input.x + matrix[4] * input.y + matrix[8] * input.z + matrix[12] * input.w,
    y : matrix[1] * input.x + matrix[5] * input.y + matrix[9] * input.z + matrix[13] * input.w,
    z : matrix[2] * input.x + matrix[6] * input.y + matrix[10] * input.z + matrix[14] * input.w,
    w : matrix[3] * input.x + matrix[7] * input.y + matrix[11] * input.z + matrix[15] * input.w,
  };
}

// Creates a unit-length quaternion.
// |input| - point-like dict (must have x, y, z, w)
let normalize_quaternion = function(input) {
  const length_squared = input.x * input.x + input.y * input.y + input.z * input.z + input.w * input.w;
  const length = Math.sqrt(length_squared);

  return {x : input.x / length, y : input.y / length, z : input.z / length, w : input.w / length};
}

// |input| - point-like dict (must have x, y, z, w)
let conjugate_quaternion = function(input) {
  return {x : -input.x, y : -input.y, z : -input.z, w : input.w};
}

let multiply_quaternions = function(q1, q2) {
  return {
    w : q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z,
    x : q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
    y : q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
    z : q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
  }
}

// |point| - point-like dict (must have x, y, z, w)
let normalize_perspective = function(point) {
  if(point.w == 0 || point.w == 1) return point;

  return {
    x : point.x / point.w,
    y : point.y / point.w,
    z : point.z / point.w,
    w : 1
  };
}

// |quaternion| - point-like dict (must have x, y, z, w),
// |input| - point-like dict (must have x, y, z, w)
let transform_point_by_quaternion = function(quaternion, input) {
  const q_normalized = normalize_quaternion(quaternion);
  const q_conj = conjugate_quaternion(q_normalized);
  const p_in = normalize_perspective(input);

  // construct a quaternion out of the point (take xyz & zero the real part).
  const p = {x : p_in.x, y : p_in.y, z : p_in.z, w : 0};

  // transform the input point
  const p_mul = multiply_quaternions( q_normalized, multiply_quaternions(p, q_conj) );

  // add back the w component of the input
  return { x : p_mul.x, y : p_mul.y, z : p_mul.z, w : p_in.w };
}
