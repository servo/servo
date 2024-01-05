/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /* Data used for multisample tests */const samplePositionToFragmentPosition = (pos) =>
pos.map((v) => v / 16);
const samplePositionsToFragmentPositions = (
positions) =>
positions.map(samplePositionToFragmentPosition);

// These are sample positions based on a 16x16 grid with 0,0 at the top left.
// For example 8,8 would be a fragment coordinate of 0.5, 0.5
// Based on: https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_standard_multisample_quality_levels
const kMultisamplingTables = new Map([
[1, samplePositionsToFragmentPositions([[8, 8]])],
[
2,
samplePositionsToFragmentPositions([
[4, 4],
[12, 12]]
)],

[
4,
samplePositionsToFragmentPositions([
[6, 2],
[14, 6],
[2, 10],
[10, 14]]
)],

[
8,
samplePositionsToFragmentPositions([
[9, 5],
[7, 11],
[13, 9],
[5, 3],
[3, 13],
[1, 7],
[11, 15],
[15, 1]]
)],

[
16,
samplePositionsToFragmentPositions([
[9, 9],
[7, 5],
[5, 10],
[12, 7],

[3, 6],
[10, 13],
[13, 11],
[11, 3],

[6, 14],
[8, 1],
[4, 2],
[2, 12],

[0, 8],
[15, 4],
[14, 15],
[1, 0]]
)]]

);

/**
 * For a given sampleCount returns an array of 2d fragment offsets
 * where each offset is between 0 and 1.
 */
export function getMultisampleFragmentOffsets(sampleCount) {
  return kMultisamplingTables.get(sampleCount);
}