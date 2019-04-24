// assert_equals can fail when comparing floats due to precision errors, so
// use assert_approx_equals with this constant instead
const FLOAT_EPSILON = 0.001;

// Identity matrix
const IDENTITY_MATRIX = [1, 0, 0, 0,
                         0, 1, 0, 0,
                         0, 0, 1, 0,
                         0, 0, 0, 1];

// A valid pose matrix for  when we don't care about specific values
const VALID_POSE_MATRIX = [0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1];

const VALID_PROJECTION_MATRIX =
    [1, 0, 0, 0, 0, 1, 0, 0, 3, 2, -1, -1, 0, 0, -0.2, 0];

const VALID_VIEW_MATRIX = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 4, 3, 2, 1];

// A valid VRPose for when we want the HMD to report being at the origin
const ORIGIN_POSE = IDENTITY_MATRIX;

// A valid input grip matrix for  when we don't care about specific values
const VALID_GRIP = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 4, 3, 2, 1];

// A valid input pointer offset for  when we don't care about specific values
const VALID_POINTER_OFFSET = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1];

const VALID_GRIP_WITH_POINTER_OFFSET =
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 4, 3, 3, 1];

const VALID_STAGE_TRANSFORM =
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1.0, 1.65, -1.0, 1];
