'use strict';

// Math helper - used mainly in hit test implementation done by webxr-test.js
class XRMathHelper {
  static toString(p) {
    return "[" + p.x + "," + p.y + "," + p.z + "," + p.w + "]";
  }

  static transform_by_matrix(matrix, point) {
    return {
      x : matrix[0] * point.x + matrix[4] * point.y + matrix[8] * point.z + matrix[12] * point.w,
      y : matrix[1] * point.x + matrix[5] * point.y + matrix[9] * point.z + matrix[13] * point.w,
      z : matrix[2] * point.x + matrix[6] * point.y + matrix[10] * point.z + matrix[14] * point.w,
      w : matrix[3] * point.x + matrix[7] * point.y + matrix[11] * point.z + matrix[15] * point.w,
    };
  }

  static neg(p) {
    return {x : -p.x, y : -p.y, z : -p.z, w : p.w};
  }

  static sub(lhs, rhs) {
    // .w is treated here like an entity type, 1 signifies points, 0 signifies vectors.
    // point - point, point - vector, vector - vector are ok, vector - point is not.
    if (lhs.w != rhs.w && lhs.w == 0.0) {
      throw new Error("vector - point not allowed: " + toString(lhs) + "-" + toString(rhs));
    }

    return {x : lhs.x - rhs.x, y : lhs.y - rhs.y, z : lhs.z - rhs.z, w : lhs.w - rhs.w};
  }

  static add(lhs, rhs) {
    if (lhs.w == rhs.w && lhs.w == 1.0) {
      throw new Error("point + point not allowed", p1, p2);
    }

    return {x : lhs.x + rhs.x, y : lhs.y + rhs.y, z : lhs.z + rhs.z, w : lhs.w + rhs.w};
  }

  static cross(lhs, rhs) {
    if (lhs.w != 0.0 || rhs.w != 0.0) {
      throw new Error("cross product not allowed: " + toString(lhs) + "x" + toString(rhs));
    }

    return {
      x : lhs.y * rhs.z - lhs.z * rhs.y,
      y : lhs.z * rhs.x - lhs.x * rhs.z,
      z : lhs.x * rhs.y - lhs.y * rhs.x,
      w : 0
    };
  }

  static dot(lhs, rhs) {
    if (lhs.w != 0 || rhs.w != 0) {
      throw new Error("dot product not allowed: " + toString(lhs) + "x" + toString(rhs));
    }

    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
  }

  static mul(scalar, vector) {
    if (vector.w != 0) {
      throw new Error("scalar * vector not allowed", scalar, vector);
    }

    return {x : vector.x * scalar, y : vector.y * scalar, z : vector.z * scalar, w : vector.w};
  }

  static length(vector) {
    return Math.sqrt(XRMathHelper.dot(vector, vector));
  }

  static normalize(vector) {
    const l = XRMathHelper.length(vector);
    return XRMathHelper.mul(1.0/l, vector);
  }

  // All |face|'s points and |point| must be co-planar.
  static pointInFace(point, face) {
    const normalize = XRMathHelper.normalize;
    const sub = XRMathHelper.sub;
    const length = XRMathHelper.length;
    const cross = XRMathHelper.cross;

    let onTheRight = null;
    let previous_point = face[face.length - 1];

    // |point| is in |face| if it's on the same side of all the edges.
    for (let i = 0; i < face.length; ++i) {
      const current_point = face[i];

      const edge_direction = normalize(sub(current_point, previous_point));
      const turn_direction = normalize(sub(point, current_point));

      const sin_turn_angle = length(cross(edge_direction, turn_direction));

      if (onTheRight == null) {
        onTheRight = sin_turn_angle >= 0;
      } else {
        if (onTheRight && sin_turn_angle < 0) return false;
        if (!onTheRight && sin_turn_angle > 0) return false;
      }

      previous_point = current_point;
    }

    return true;
  }

  static det2x2(m00, m01, m10, m11) {
    return m00 * m11 - m01 * m10;
  }

  static det3x3(
    m00, m01, m02,
    m10, m11, m12,
    m20, m21, m22
  ){
    const det2x2 = XRMathHelper.det2x2;

    return    m00 * det2x2(m11, m12, m21, m22)
            - m01 * det2x2(m10, m12, m20, m22)
            + m02 * det2x2(m10, m11, m20, m21);
  }

  static det4x4(
    m00, m01, m02, m03,
    m10, m11, m12, m13,
    m20, m21, m22, m23,
    m30, m31, m32, m33
  ) {
    const det3x3 = XRMathHelper.det3x3;

    return  m00 * det3x3(m11, m12, m13,
                         m21, m22, m23,
                         m31, m32, m33)
          - m01 * det3x3(m10, m12, m13,
                         m20, m22, m23,
                         m30, m32, m33)
          + m02 * det3x3(m10, m11, m13,
                         m20, m21, m23,
                         m30, m31, m33)
          - m03 * det3x3(m10, m11, m12,
                         m20, m21, m22,
                         m30, m31, m32);
  }

  static inv2(m) {
    // mij - i-th column, j-th row
    const m00 = m[0],  m01 = m[1],  m02 = m[2],  m03 = m[3];
    const m10 = m[4],  m11 = m[5],  m12 = m[6],  m13 = m[7];
    const m20 = m[8],  m21 = m[9],  m22 = m[10], m23 = m[11];
    const m30 = m[12], m31 = m[13], m32 = m[14], m33 = m[15];

    const det = det4x4(
      m00, m01, m02, m03,
      m10, m11, m12, m13,
      m20, m21, m22, m23,
      m30, m31, m32, m33
    );
  }

  static transpose(m) {
    const result = Array(16);
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        result[i * 4 + j] = m[j * 4 + i];
      }
    }
    return result;
  }

  // Inverts the matrix, ported from transformation_matrix.cc.
  static inverse(m) {
    const det3x3 = XRMathHelper.det3x3;

    // mij - i-th column, j-th row
    const m00 = m[0],  m01 = m[1],  m02 = m[2],  m03 = m[3];
    const m10 = m[4],  m11 = m[5],  m12 = m[6],  m13 = m[7];
    const m20 = m[8],  m21 = m[9],  m22 = m[10], m23 = m[11];
    const m30 = m[12], m31 = m[13], m32 = m[14], m33 = m[15];

    const det = XRMathHelper.det4x4(
      m00, m01, m02, m03,
      m10, m11, m12, m13,
      m20, m21, m22, m23,
      m30, m31, m32, m33
    );

    if (Math.abs(det) < 0.0001) {
      return null;
    }

    const invDet = 1.0 / det;
    // Calculate `comatrix * 1/det`:
    const result2 = [
      // First column (m0r):
      invDet * det3x3(m11, m12, m13, m21, m22, m23, m32, m32, m33),
      -invDet * det3x3(m10, m12, m13, m20, m22, m23, m30, m32, m33),
      invDet * det3x3(m10, m11, m13, m20, m21, m23, m30, m31, m33),
      -invDet * det3x3(m10, m11, m12, m20, m21, m22, m30, m31, m32),
      // Second column (m1r):
      -invDet * det3x3(m01, m02, m03, m21, m22, m23, m32, m32, m33),
      invDet * det3x3(m00, m02, m03, m20, m22, m23, m30, m32, m33),
      -invDet * det3x3(m00, m01, m03, m20, m21, m23, m30, m31, m33),
      invDet * det3x3(m00, m01, m02, m20, m21, m22, m30, m31, m32),
      // Third column (m2r):
      invDet * det3x3(m01, m02, m03, m11, m12, m13, m31, m32, m33),
      -invDet * det3x3(m00, m02, m03, m10, m12, m13, m30, m32, m33),
      invDet * det3x3(m00, m01, m03, m10, m11, m13, m30, m31, m33),
      -invDet * det3x3(m00, m01, m02, m10, m11, m12, m30, m31, m32),
      // Fourth column (m3r):
      -invDet * det3x3(m01, m02, m03, m11, m12, m13, m21, m22, m23),
      invDet * det3x3(m00, m02, m03, m10, m12, m13, m20, m22, m23),
      -invDet * det3x3(m00, m01, m03, m10, m11, m13, m20, m21, m23),
      invDet * det3x3(m00, m01, m02, m10, m11, m12, m20, m21, m22),
    ];

    // Actual inverse is `1/det * transposed(comatrix)`:
    return XRMathHelper.transpose(result2);
  }

  static mul4x4(m1, m2) {
    if (m1 == null || m2 == null) {
      return null;
    }

    const result = Array(16);

    for (let row = 0; row < 4; row++) {
      for (let col = 0; col < 4; col++) {
        result[4 * col + row] = 0;
        for(let i = 0; i < 4; i++) {
          result[4 * col + row] += m1[4 * i + row] * m2[4 * col + i];
        }
      }
    }

    return result;
  }

  static identity() {
    return [
      1, 0, 0, 0,
      0, 1, 0, 0,
      0, 0, 1, 0,
      0, 0, 0, 1
    ];
  };
}

XRMathHelper.EPSILON = 0.001;
