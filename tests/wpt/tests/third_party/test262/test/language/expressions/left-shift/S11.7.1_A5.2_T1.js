// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x << y uses ToUint32(AdditiveExpression) & 31
es5id: 11.7.1_A5.2_T1
description: Checking distinct points
---*/

//CHECK#1
if (1 << -32.1 !== 1) { 
  throw new Test262Error('#1: 1 << -32.1 === 1. Actual: ' + (1 << -32.1)); 
} 

//CHECK#2
if (1 << -31.1 !== 2) { 
  throw new Test262Error('#2: 1 << -31.1 === 2. Actual: ' + (1 << -31.1)); 
} 

//CHECK#3
if (1 << -30.1 !== 4) { 
  throw new Test262Error('#3: 1 << -30.1 === 4. Actual: ' + (1 << -30.1)); 
} 

//CHECK#4
if (1 << -29.1 !== 8) { 
  throw new Test262Error('#4: 1 << -29.1 === 8. Actual: ' + (1 << -29.1)); 
} 

//CHECK#5
if (1 << -28.1 !== 16) { 
  throw new Test262Error('#5: 1 << -28.1 === 16. Actual: ' + (1 << -28.1)); 
} 

//CHECK#6
if (1 << -27.1 !== 32) { 
  throw new Test262Error('#6: 1 << -27.1 === 32. Actual: ' + (1 << -27.1)); 
} 

//CHECK#7
if (1 << -26.1 !== 64) { 
  throw new Test262Error('#7: 1 << -26.1 === 64. Actual: ' + (1 << -26.1)); 
} 

//CHECK#8
if (1 << -25.1 !== 128) { 
  throw new Test262Error('#8: 1 << -25.1 === 128. Actual: ' + (1 << -25.1)); 
} 

//CHECK#9
if (1 << -24.1 !== 256) { 
  throw new Test262Error('#9: 1 << -24.1 === 256. Actual: ' + (1 << -24.1)); 
} 

//CHECK#10
if (1 << -23.1 !== 512) { 
  throw new Test262Error('#10: 1 << -23.1 === 512. Actual: ' + (1 << -23.1)); 
} 

//CHECK#11
if (1 << -22.1 !== 1024) { 
  throw new Test262Error('#11: 1 << -22.1 === 1024. Actual: ' + (1 << -22.1)); 
} 

//CHECK#12
if (1 << -21.1 !== 2048) { 
  throw new Test262Error('#12: 1 << -21.1 === 2048. Actual: ' + (1 << -21.1)); 
} 

//CHECK#13
if (1 << -20.1 !== 4096) { 
  throw new Test262Error('#13: 1 << -20.1 === 4096. Actual: ' + (1 << -20.1)); 
} 

//CHECK#14
if (1 << -19.1 !== 8192) { 
  throw new Test262Error('#14: 1 << -19.1 === 8192. Actual: ' + (1 << -19.1)); 
} 

//CHECK#15
if (1 << -18.1 !== 16384) { 
  throw new Test262Error('#15: 1 << -18.1 === 16384. Actual: ' + (1 << -18.1)); 
} 

//CHECK#16
if (1 << -17.1 !== 32768) { 
  throw new Test262Error('#16: 1 << -17.1 === 32768. Actual: ' + (1 << -17.1)); 
} 

//CHECK#17
if (1 << -16.1 !== 65536) { 
  throw new Test262Error('#17: 1 << -16.1 === 65536. Actual: ' + (1 << -16.1)); 
} 

//CHECK#18
if (1 << -15.1 !== 131072) { 
  throw new Test262Error('#18: 1 << -15.1 === 131072. Actual: ' + (1 << -15.1)); 
} 

//CHECK#19
if (1 << -14.1 !== 262144) { 
  throw new Test262Error('#19: 1 << -14.1 === 262144. Actual: ' + (1 << -14.1)); 
} 

//CHECK#20
if (1 << -13.1 !== 524288) { 
  throw new Test262Error('#20: 1 << -13.1 === 524288. Actual: ' + (1 << -13.1)); 
} 

//CHECK#21
if (1 << -12.1 !== 1048576) { 
  throw new Test262Error('#21: 1 << -12.1 === 1048576. Actual: ' + (1 << -12.1)); 
} 

//CHECK#22
if (1 << -11.1 !== 2097152) { 
  throw new Test262Error('#22: 1 << -11.1 === 2097152. Actual: ' + (1 << -11.1)); 
} 

//CHECK#23
if (1 << -10.1 !== 4194304) { 
  throw new Test262Error('#23: 1 << -10.1 === 4194304. Actual: ' + (1 << -10.1)); 
} 

//CHECK#24
if (1 << -9.1 !== 8388608) { 
  throw new Test262Error('#24: 1 << -9.1 === 8388608. Actual: ' + (1 << -9.1)); 
} 

//CHECK#25
if (1 << -8.1 !== 16777216) { 
  throw new Test262Error('#25: 1 << -8.1 === 16777216. Actual: ' + (1 << -8.1)); 
} 

//CHECK#26
if (1 << -7.1 !== 33554432) { 
  throw new Test262Error('#26: 1 << -7.1 === 33554432. Actual: ' + (1 << -7.1)); 
} 

//CHECK#27
if (1 << -6.1 !== 67108864) { 
  throw new Test262Error('#27: 1 << -6.1 === 67108864. Actual: ' + (1 << -6.1)); 
} 

//CHECK#28
if (1 << -5.1 !== 134217728) { 
  throw new Test262Error('#28: 1 << -5.1 === 134217728. Actual: ' + (1 << -5.1)); 
} 

//CHECK#29
if (1 << -4.1 !== 268435456) { 
  throw new Test262Error('#29: 1 << -4.1 === 268435456. Actual: ' + (1 << -4.1)); 
} 

//CHECK#30
if (1 << -3.1 !== 536870912) { 
  throw new Test262Error('#30: 1 << -3.1 === 536870912. Actual: ' + (1 << -3.1)); 
} 

//CHECK#31
if (1 << -2.1 !== 1073741824) { 
  throw new Test262Error('#31: 1 << -2.1 === 1073741824. Actual: ' + (1 << -2.1)); 
} 

//CHECK#32
if (1 << -1.1 !== -2147483648) { 
  throw new Test262Error('#32: 1 << -1.1 === -2147483648. Actual: ' + (1 << -1.1)); 
} 

//CHECK#33
if (1 << 32.1 !== 1) { 
  throw new Test262Error('#33: 1 << 32.1 === 1. Actual: ' + (1 << 32.1)); 
} 

//CHECK#34
if (1 << 33.1 !== 2) { 
  throw new Test262Error('#34: 1 << 33.1 === 2. Actual: ' + (1 << 33.1)); 
} 

//CHECK#35
if (1 << 34.1 !== 4) { 
  throw new Test262Error('#35: 1 << 34.1 === 4. Actual: ' + (1 << 34.1)); 
} 

//CHECK#36
if (1 << 35.1 !== 8) { 
  throw new Test262Error('#36: 1 << 35.1 === 8. Actual: ' + (1 << 35.1)); 
} 

//CHECK#37
if (1 << 36.1 !== 16) { 
  throw new Test262Error('#37: 1 << 36.1 === 16. Actual: ' + (1 << 36.1)); 
} 

//CHECK#38
if (1 << 37.1 !== 32) { 
  throw new Test262Error('#38: 1 << 37.1 === 32. Actual: ' + (1 << 37.1)); 
} 

//CHECK#39
if (1 << 38.1 !== 64) { 
  throw new Test262Error('#39: 1 << 38.1 === 64. Actual: ' + (1 << 38.1)); 
} 

//CHECK#40
if (1 << 39.1 !== 128) { 
  throw new Test262Error('#40: 1 << 39.1 === 128. Actual: ' + (1 << 39.1)); 
} 

//CHECK#41
if (1 << 40.1 !== 256) { 
  throw new Test262Error('#41: 1 << 40.1 === 256. Actual: ' + (1 << 40.1)); 
} 

//CHECK#42
if (1 << 41.1 !== 512) { 
  throw new Test262Error('#42: 1 << 41.1 === 512. Actual: ' + (1 << 41.1)); 
} 

//CHECK#43
if (1 << 42.1 !== 1024) { 
  throw new Test262Error('#43: 1 << 42.1 === 1024. Actual: ' + (1 << 42.1)); 
} 

//CHECK#44
if (1 << 43.1 !== 2048) { 
  throw new Test262Error('#44: 1 << 43.1 === 2048. Actual: ' + (1 << 43.1)); 
} 

//CHECK#45
if (1 << 44.1 !== 4096) { 
  throw new Test262Error('#45: 1 << 44.1 === 4096. Actual: ' + (1 << 44.1)); 
} 

//CHECK#46
if (1 << 45.1 !== 8192) { 
  throw new Test262Error('#46: 1 << 45.1 === 8192. Actual: ' + (1 << 45.1)); 
} 

//CHECK#47
if (1 << 46.1 !== 16384) { 
  throw new Test262Error('#47: 1 << 46.1 === 16384. Actual: ' + (1 << 46.1)); 
} 

//CHECK#48
if (1 << 47.1 !== 32768) { 
  throw new Test262Error('#48: 1 << 47.1 === 32768. Actual: ' + (1 << 47.1)); 
} 

//CHECK#49
if (1 << 48.1 !== 65536) { 
  throw new Test262Error('#49: 1 << 48.1 === 65536. Actual: ' + (1 << 48.1)); 
} 

//CHECK#50
if (1 << 49.1 !== 131072) { 
  throw new Test262Error('#50: 1 << 49.1 === 131072. Actual: ' + (1 << 49.1)); 
} 

//CHECK#51
if (1 << 50.1 !== 262144) { 
  throw new Test262Error('#51: 1 << 50.1 === 262144. Actual: ' + (1 << 50.1)); 
} 

//CHECK#52
if (1 << 51.1 !== 524288) { 
  throw new Test262Error('#52: 1 << 51.1 === 524288. Actual: ' + (1 << 51.1)); 
} 

//CHECK#53
if (1 << 52.1 !== 1048576) { 
  throw new Test262Error('#53: 1 << 52.1 === 1048576. Actual: ' + (1 << 52.1)); 
} 

//CHECK#54
if (1 << 53.1 !== 2097152) { 
  throw new Test262Error('#54: 1 << 53.1 === 2097152. Actual: ' + (1 << 53.1)); 
} 

//CHECK#55
if (1 << 54.1 !== 4194304) { 
  throw new Test262Error('#55: 1 << 54.1 === 4194304. Actual: ' + (1 << 54.1)); 
} 

//CHECK#56
if (1 << 55.1 !== 8388608) { 
  throw new Test262Error('#56: 1 << 55.1 === 8388608. Actual: ' + (1 << 55.1)); 
} 

//CHECK#57
if (1 << 56.1 !== 16777216) { 
  throw new Test262Error('#57: 1 << 56.1 === 16777216. Actual: ' + (1 << 56.1)); 
} 

//CHECK#58
if (1 << 57.1 !== 33554432) { 
  throw new Test262Error('#58: 1 << 57.1 === 33554432. Actual: ' + (1 << 57.1)); 
} 

//CHECK#59
if (1 << 58.1 !== 67108864) { 
  throw new Test262Error('#59: 1 << 58.1 === 67108864. Actual: ' + (1 << 58.1)); 
} 

//CHECK#60
if (1 << 59.1 !== 134217728) { 
  throw new Test262Error('#60: 1 << 59.1 === 134217728. Actual: ' + (1 << 59.1)); 
} 

//CHECK#61
if (1 << 60.1 !== 268435456) { 
  throw new Test262Error('#61: 1 << 60.1 === 268435456. Actual: ' + (1 << 60.1)); 
} 

//CHECK#62
if (1 << 61.1 !== 536870912) { 
  throw new Test262Error('#62: 1 << 61.1 === 536870912. Actual: ' + (1 << 61.1)); 
} 

//CHECK#63
if (1 << 62.1 !== 1073741824) { 
  throw new Test262Error('#63: 1 << 62.1 === 1073741824. Actual: ' + (1 << 62.1)); 
} 

//CHECK#64
if (1 << 63.1 !== -2147483648) { 
  throw new Test262Error('#64: 1 << 63.1 === -2147483648. Actual: ' + (1 << 63.1)); 
}
