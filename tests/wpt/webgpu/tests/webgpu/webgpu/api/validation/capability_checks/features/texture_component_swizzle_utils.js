/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/




// Note: There are 4 settings with 6 options which is 1296 combinations. So we don't check them all. Just a few below.
export const kSwizzleTests = [
'rgba',
'0000',
'1111',
'rrrr',
'gggg',
'bbbb',
'aaaa',
'abgr',
'gbar',
'barg',
'argb',
'0gba',
'r0ba',
'rg0a',
'rgb0',
'1gba',
'r1ba',
'rg1a',
'rgb1'];



// Returns true if swizzle is identity
export function isIdentitySwizzle(swizzle) {
  return swizzle === undefined || swizzle === 'rgba';
}