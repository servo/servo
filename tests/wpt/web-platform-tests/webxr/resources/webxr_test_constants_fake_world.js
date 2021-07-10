'use strict';

// This file introduces constants used to mock fake world for the purposes of hit test.

// Generates FakeXRWorldInit dictionary with given dimensions.
// The generated fake world will have floor and front wall treated as planes,
// side walls treated as meshes, and ceiling treated as points.
// width - X axis, in meters
// height - Y axis, in meters
// length - Z axis, in meters
function createFakeWorld(
  width, height, length,
  front_wall_and_floor_type = "plane",
  side_walls_type = "mesh",
  ceiling_type = "point") {
  // Vertices:
  const BOTTOM_LEFT_FRONT = { x: -width/2, y: 0, z: -length/2, w: 1};
  const BOTTOM_RIGHT_FRONT = { x: width/2, y: 0, z: -length/2, w: 1};

  const TOP_LEFT_FRONT = { x: -width/2, y: height, z: -length/2, w: 1};
  const TOP_RIGHT_FRONT = { x: width/2, y: height, z: -length/2, w: 1};

  const BOTTOM_LEFT_BACK = { x: -width/2, y: 0, z: length/2, w: 1};
  const BOTTOM_RIGHT_BACK = { x: width/2, y: 0, z: length/2, w: 1};

  const TOP_LEFT_BACK = { x: -width/2, y: height, z: length/2, w: 1};
  const TOP_RIGHT_BACK = { x: width/2, y: height, z: length/2, w: 1};

  // Faces:
  const FRONT_WALL_AND_FLOOR_FACES = [
    // Front wall:
    { vertices: [BOTTOM_LEFT_FRONT, BOTTOM_RIGHT_FRONT, TOP_RIGHT_FRONT] },
    { vertices: [BOTTOM_LEFT_FRONT, TOP_RIGHT_FRONT, TOP_LEFT_FRONT] },
    // Floor:
    { vertices: [BOTTOM_LEFT_FRONT, BOTTOM_RIGHT_FRONT, BOTTOM_RIGHT_BACK] },
    { vertices: [BOTTOM_LEFT_FRONT, BOTTOM_LEFT_BACK, BOTTOM_RIGHT_BACK] },
  ];

  const CEILING_FACES = [
    // Ceiling:
    { vertices: [TOP_LEFT_FRONT, TOP_RIGHT_FRONT, TOP_RIGHT_BACK] },
    { vertices: [TOP_LEFT_FRONT, TOP_LEFT_BACK, TOP_RIGHT_BACK] },
  ];

  const SIDE_WALLS_FACES = [
    // Left:
    { vertices: [BOTTOM_LEFT_FRONT, TOP_LEFT_FRONT, TOP_LEFT_BACK] },
    { vertices: [BOTTOM_LEFT_FRONT, BOTTOM_LEFT_BACK, TOP_LEFT_BACK] },
    // Right:
    { vertices: [BOTTOM_RIGHT_FRONT, TOP_RIGHT_FRONT, TOP_RIGHT_BACK] },
    { vertices: [BOTTOM_RIGHT_FRONT, BOTTOM_RIGHT_BACK, TOP_RIGHT_BACK] },
  ];

  // Regions:
  const FRONT_WALL_AND_FLOOR_REGION = {
    type: front_wall_and_floor_type,
    faces: FRONT_WALL_AND_FLOOR_FACES,
  };

  const SIDE_WALLS_REGION = {
    type: side_walls_type,
    faces: SIDE_WALLS_FACES,
  };

  const CEILING_REGION = {
    type: ceiling_type,
    faces: CEILING_FACES,
  };

  return {
    hitTestRegions : [
      FRONT_WALL_AND_FLOOR_REGION,
      SIDE_WALLS_REGION,
      CEILING_REGION,
    ]
  };
}

const VALID_FAKE_WORLD = createFakeWorld(5.0, 2.0, 5.0);
