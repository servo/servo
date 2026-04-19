// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check x >> y operator in distinct points
es5id: 11.7.2_A4_T4
description: ShiftExpression = 2^n - 1, n = 16...31
---*/

//CHECK
 
if (0 >> 16 !== 0) { 
  throw new Test262Error('#513: 0 >> 16 === 0. Actual: ' + (0 >> 16)); 
} 


if (1 >> 16 !== 0) { 
  throw new Test262Error('#514: 1 >> 16 === 0. Actual: ' + (1 >> 16)); 
} 


if (3 >> 16 !== 0) { 
  throw new Test262Error('#515: 3 >> 16 === 0. Actual: ' + (3 >> 16)); 
} 


if (7 >> 16 !== 0) { 
  throw new Test262Error('#516: 7 >> 16 === 0. Actual: ' + (7 >> 16)); 
} 


if (15 >> 16 !== 0) { 
  throw new Test262Error('#517: 15 >> 16 === 0. Actual: ' + (15 >> 16)); 
} 


if (31 >> 16 !== 0) { 
  throw new Test262Error('#518: 31 >> 16 === 0. Actual: ' + (31 >> 16)); 
} 


if (63 >> 16 !== 0) { 
  throw new Test262Error('#519: 63 >> 16 === 0. Actual: ' + (63 >> 16)); 
} 


if (127 >> 16 !== 0) { 
  throw new Test262Error('#520: 127 >> 16 === 0. Actual: ' + (127 >> 16)); 
} 


if (255 >> 16 !== 0) { 
  throw new Test262Error('#521: 255 >> 16 === 0. Actual: ' + (255 >> 16)); 
} 


if (511 >> 16 !== 0) { 
  throw new Test262Error('#522: 511 >> 16 === 0. Actual: ' + (511 >> 16)); 
} 


if (1023 >> 16 !== 0) { 
  throw new Test262Error('#523: 1023 >> 16 === 0. Actual: ' + (1023 >> 16)); 
} 


if (2047 >> 16 !== 0) { 
  throw new Test262Error('#524: 2047 >> 16 === 0. Actual: ' + (2047 >> 16)); 
} 


if (4095 >> 16 !== 0) { 
  throw new Test262Error('#525: 4095 >> 16 === 0. Actual: ' + (4095 >> 16)); 
} 


if (8191 >> 16 !== 0) { 
  throw new Test262Error('#526: 8191 >> 16 === 0. Actual: ' + (8191 >> 16)); 
} 


if (16383 >> 16 !== 0) { 
  throw new Test262Error('#527: 16383 >> 16 === 0. Actual: ' + (16383 >> 16)); 
} 


if (32767 >> 16 !== 0) { 
  throw new Test262Error('#528: 32767 >> 16 === 0. Actual: ' + (32767 >> 16)); 
} 


if (65535 >> 16 !== 0) { 
  throw new Test262Error('#529: 65535 >> 16 === 0. Actual: ' + (65535 >> 16)); 
} 


if (131071 >> 16 !== 1) { 
  throw new Test262Error('#530: 131071 >> 16 === 1. Actual: ' + (131071 >> 16)); 
} 


if (262143 >> 16 !== 3) { 
  throw new Test262Error('#531: 262143 >> 16 === 3. Actual: ' + (262143 >> 16)); 
} 


if (524287 >> 16 !== 7) { 
  throw new Test262Error('#532: 524287 >> 16 === 7. Actual: ' + (524287 >> 16)); 
} 


if (1048575 >> 16 !== 15) { 
  throw new Test262Error('#533: 1048575 >> 16 === 15. Actual: ' + (1048575 >> 16)); 
} 


if (2097151 >> 16 !== 31) { 
  throw new Test262Error('#534: 2097151 >> 16 === 31. Actual: ' + (2097151 >> 16)); 
} 


if (4194303 >> 16 !== 63) { 
  throw new Test262Error('#535: 4194303 >> 16 === 63. Actual: ' + (4194303 >> 16)); 
} 


if (8388607 >> 16 !== 127) { 
  throw new Test262Error('#536: 8388607 >> 16 === 127. Actual: ' + (8388607 >> 16)); 
} 


if (16777215 >> 16 !== 255) { 
  throw new Test262Error('#537: 16777215 >> 16 === 255. Actual: ' + (16777215 >> 16)); 
} 


if (33554431 >> 16 !== 511) { 
  throw new Test262Error('#538: 33554431 >> 16 === 511. Actual: ' + (33554431 >> 16)); 
} 


if (67108863 >> 16 !== 1023) { 
  throw new Test262Error('#539: 67108863 >> 16 === 1023. Actual: ' + (67108863 >> 16)); 
} 


if (134217727 >> 16 !== 2047) { 
  throw new Test262Error('#540: 134217727 >> 16 === 2047. Actual: ' + (134217727 >> 16)); 
} 


if (268435455 >> 16 !== 4095) { 
  throw new Test262Error('#541: 268435455 >> 16 === 4095. Actual: ' + (268435455 >> 16)); 
} 


if (536870911 >> 16 !== 8191) { 
  throw new Test262Error('#542: 536870911 >> 16 === 8191. Actual: ' + (536870911 >> 16)); 
} 


if (1073741823 >> 16 !== 16383) { 
  throw new Test262Error('#543: 1073741823 >> 16 === 16383. Actual: ' + (1073741823 >> 16)); 
} 


if (2147483647 >> 16 !== 32767) { 
  throw new Test262Error('#544: 2147483647 >> 16 === 32767. Actual: ' + (2147483647 >> 16)); 
} 


if (0 >> 17 !== 0) { 
  throw new Test262Error('#545: 0 >> 17 === 0. Actual: ' + (0 >> 17)); 
} 


if (1 >> 17 !== 0) { 
  throw new Test262Error('#546: 1 >> 17 === 0. Actual: ' + (1 >> 17)); 
} 


if (3 >> 17 !== 0) { 
  throw new Test262Error('#547: 3 >> 17 === 0. Actual: ' + (3 >> 17)); 
} 


if (7 >> 17 !== 0) { 
  throw new Test262Error('#548: 7 >> 17 === 0. Actual: ' + (7 >> 17)); 
} 


if (15 >> 17 !== 0) { 
  throw new Test262Error('#549: 15 >> 17 === 0. Actual: ' + (15 >> 17)); 
} 


if (31 >> 17 !== 0) { 
  throw new Test262Error('#550: 31 >> 17 === 0. Actual: ' + (31 >> 17)); 
} 


if (63 >> 17 !== 0) { 
  throw new Test262Error('#551: 63 >> 17 === 0. Actual: ' + (63 >> 17)); 
} 


if (127 >> 17 !== 0) { 
  throw new Test262Error('#552: 127 >> 17 === 0. Actual: ' + (127 >> 17)); 
} 


if (255 >> 17 !== 0) { 
  throw new Test262Error('#553: 255 >> 17 === 0. Actual: ' + (255 >> 17)); 
} 


if (511 >> 17 !== 0) { 
  throw new Test262Error('#554: 511 >> 17 === 0. Actual: ' + (511 >> 17)); 
} 


if (1023 >> 17 !== 0) { 
  throw new Test262Error('#555: 1023 >> 17 === 0. Actual: ' + (1023 >> 17)); 
} 


if (2047 >> 17 !== 0) { 
  throw new Test262Error('#556: 2047 >> 17 === 0. Actual: ' + (2047 >> 17)); 
} 


if (4095 >> 17 !== 0) { 
  throw new Test262Error('#557: 4095 >> 17 === 0. Actual: ' + (4095 >> 17)); 
} 


if (8191 >> 17 !== 0) { 
  throw new Test262Error('#558: 8191 >> 17 === 0. Actual: ' + (8191 >> 17)); 
} 


if (16383 >> 17 !== 0) { 
  throw new Test262Error('#559: 16383 >> 17 === 0. Actual: ' + (16383 >> 17)); 
} 


if (32767 >> 17 !== 0) { 
  throw new Test262Error('#560: 32767 >> 17 === 0. Actual: ' + (32767 >> 17)); 
} 


if (65535 >> 17 !== 0) { 
  throw new Test262Error('#561: 65535 >> 17 === 0. Actual: ' + (65535 >> 17)); 
} 


if (131071 >> 17 !== 0) { 
  throw new Test262Error('#562: 131071 >> 17 === 0. Actual: ' + (131071 >> 17)); 
} 


if (262143 >> 17 !== 1) { 
  throw new Test262Error('#563: 262143 >> 17 === 1. Actual: ' + (262143 >> 17)); 
} 


if (524287 >> 17 !== 3) { 
  throw new Test262Error('#564: 524287 >> 17 === 3. Actual: ' + (524287 >> 17)); 
} 


if (1048575 >> 17 !== 7) { 
  throw new Test262Error('#565: 1048575 >> 17 === 7. Actual: ' + (1048575 >> 17)); 
} 


if (2097151 >> 17 !== 15) { 
  throw new Test262Error('#566: 2097151 >> 17 === 15. Actual: ' + (2097151 >> 17)); 
} 


if (4194303 >> 17 !== 31) { 
  throw new Test262Error('#567: 4194303 >> 17 === 31. Actual: ' + (4194303 >> 17)); 
} 


if (8388607 >> 17 !== 63) { 
  throw new Test262Error('#568: 8388607 >> 17 === 63. Actual: ' + (8388607 >> 17)); 
} 


if (16777215 >> 17 !== 127) { 
  throw new Test262Error('#569: 16777215 >> 17 === 127. Actual: ' + (16777215 >> 17)); 
} 


if (33554431 >> 17 !== 255) { 
  throw new Test262Error('#570: 33554431 >> 17 === 255. Actual: ' + (33554431 >> 17)); 
} 


if (67108863 >> 17 !== 511) { 
  throw new Test262Error('#571: 67108863 >> 17 === 511. Actual: ' + (67108863 >> 17)); 
} 


if (134217727 >> 17 !== 1023) { 
  throw new Test262Error('#572: 134217727 >> 17 === 1023. Actual: ' + (134217727 >> 17)); 
} 


if (268435455 >> 17 !== 2047) { 
  throw new Test262Error('#573: 268435455 >> 17 === 2047. Actual: ' + (268435455 >> 17)); 
} 


if (536870911 >> 17 !== 4095) { 
  throw new Test262Error('#574: 536870911 >> 17 === 4095. Actual: ' + (536870911 >> 17)); 
} 


if (1073741823 >> 17 !== 8191) { 
  throw new Test262Error('#575: 1073741823 >> 17 === 8191. Actual: ' + (1073741823 >> 17)); 
} 


if (2147483647 >> 17 !== 16383) { 
  throw new Test262Error('#576: 2147483647 >> 17 === 16383. Actual: ' + (2147483647 >> 17)); 
} 


if (0 >> 18 !== 0) { 
  throw new Test262Error('#577: 0 >> 18 === 0. Actual: ' + (0 >> 18)); 
} 


if (1 >> 18 !== 0) { 
  throw new Test262Error('#578: 1 >> 18 === 0. Actual: ' + (1 >> 18)); 
} 


if (3 >> 18 !== 0) { 
  throw new Test262Error('#579: 3 >> 18 === 0. Actual: ' + (3 >> 18)); 
} 


if (7 >> 18 !== 0) { 
  throw new Test262Error('#580: 7 >> 18 === 0. Actual: ' + (7 >> 18)); 
} 


if (15 >> 18 !== 0) { 
  throw new Test262Error('#581: 15 >> 18 === 0. Actual: ' + (15 >> 18)); 
} 


if (31 >> 18 !== 0) { 
  throw new Test262Error('#582: 31 >> 18 === 0. Actual: ' + (31 >> 18)); 
} 


if (63 >> 18 !== 0) { 
  throw new Test262Error('#583: 63 >> 18 === 0. Actual: ' + (63 >> 18)); 
} 


if (127 >> 18 !== 0) { 
  throw new Test262Error('#584: 127 >> 18 === 0. Actual: ' + (127 >> 18)); 
} 


if (255 >> 18 !== 0) { 
  throw new Test262Error('#585: 255 >> 18 === 0. Actual: ' + (255 >> 18)); 
} 


if (511 >> 18 !== 0) { 
  throw new Test262Error('#586: 511 >> 18 === 0. Actual: ' + (511 >> 18)); 
} 


if (1023 >> 18 !== 0) { 
  throw new Test262Error('#587: 1023 >> 18 === 0. Actual: ' + (1023 >> 18)); 
} 


if (2047 >> 18 !== 0) { 
  throw new Test262Error('#588: 2047 >> 18 === 0. Actual: ' + (2047 >> 18)); 
} 


if (4095 >> 18 !== 0) { 
  throw new Test262Error('#589: 4095 >> 18 === 0. Actual: ' + (4095 >> 18)); 
} 


if (8191 >> 18 !== 0) { 
  throw new Test262Error('#590: 8191 >> 18 === 0. Actual: ' + (8191 >> 18)); 
} 


if (16383 >> 18 !== 0) { 
  throw new Test262Error('#591: 16383 >> 18 === 0. Actual: ' + (16383 >> 18)); 
} 


if (32767 >> 18 !== 0) { 
  throw new Test262Error('#592: 32767 >> 18 === 0. Actual: ' + (32767 >> 18)); 
} 


if (65535 >> 18 !== 0) { 
  throw new Test262Error('#593: 65535 >> 18 === 0. Actual: ' + (65535 >> 18)); 
} 


if (131071 >> 18 !== 0) { 
  throw new Test262Error('#594: 131071 >> 18 === 0. Actual: ' + (131071 >> 18)); 
} 


if (262143 >> 18 !== 0) { 
  throw new Test262Error('#595: 262143 >> 18 === 0. Actual: ' + (262143 >> 18)); 
} 


if (524287 >> 18 !== 1) { 
  throw new Test262Error('#596: 524287 >> 18 === 1. Actual: ' + (524287 >> 18)); 
} 


if (1048575 >> 18 !== 3) { 
  throw new Test262Error('#597: 1048575 >> 18 === 3. Actual: ' + (1048575 >> 18)); 
} 


if (2097151 >> 18 !== 7) { 
  throw new Test262Error('#598: 2097151 >> 18 === 7. Actual: ' + (2097151 >> 18)); 
} 


if (4194303 >> 18 !== 15) { 
  throw new Test262Error('#599: 4194303 >> 18 === 15. Actual: ' + (4194303 >> 18)); 
} 


if (8388607 >> 18 !== 31) { 
  throw new Test262Error('#600: 8388607 >> 18 === 31. Actual: ' + (8388607 >> 18)); 
} 


if (16777215 >> 18 !== 63) { 
  throw new Test262Error('#601: 16777215 >> 18 === 63. Actual: ' + (16777215 >> 18)); 
} 


if (33554431 >> 18 !== 127) { 
  throw new Test262Error('#602: 33554431 >> 18 === 127. Actual: ' + (33554431 >> 18)); 
} 


if (67108863 >> 18 !== 255) { 
  throw new Test262Error('#603: 67108863 >> 18 === 255. Actual: ' + (67108863 >> 18)); 
} 


if (134217727 >> 18 !== 511) { 
  throw new Test262Error('#604: 134217727 >> 18 === 511. Actual: ' + (134217727 >> 18)); 
} 


if (268435455 >> 18 !== 1023) { 
  throw new Test262Error('#605: 268435455 >> 18 === 1023. Actual: ' + (268435455 >> 18)); 
} 


if (536870911 >> 18 !== 2047) { 
  throw new Test262Error('#606: 536870911 >> 18 === 2047. Actual: ' + (536870911 >> 18)); 
} 


if (1073741823 >> 18 !== 4095) { 
  throw new Test262Error('#607: 1073741823 >> 18 === 4095. Actual: ' + (1073741823 >> 18)); 
} 


if (2147483647 >> 18 !== 8191) { 
  throw new Test262Error('#608: 2147483647 >> 18 === 8191. Actual: ' + (2147483647 >> 18)); 
} 


if (0 >> 19 !== 0) { 
  throw new Test262Error('#609: 0 >> 19 === 0. Actual: ' + (0 >> 19)); 
} 


if (1 >> 19 !== 0) { 
  throw new Test262Error('#610: 1 >> 19 === 0. Actual: ' + (1 >> 19)); 
} 


if (3 >> 19 !== 0) { 
  throw new Test262Error('#611: 3 >> 19 === 0. Actual: ' + (3 >> 19)); 
} 


if (7 >> 19 !== 0) { 
  throw new Test262Error('#612: 7 >> 19 === 0. Actual: ' + (7 >> 19)); 
} 


if (15 >> 19 !== 0) { 
  throw new Test262Error('#613: 15 >> 19 === 0. Actual: ' + (15 >> 19)); 
} 


if (31 >> 19 !== 0) { 
  throw new Test262Error('#614: 31 >> 19 === 0. Actual: ' + (31 >> 19)); 
} 


if (63 >> 19 !== 0) { 
  throw new Test262Error('#615: 63 >> 19 === 0. Actual: ' + (63 >> 19)); 
} 


if (127 >> 19 !== 0) { 
  throw new Test262Error('#616: 127 >> 19 === 0. Actual: ' + (127 >> 19)); 
} 


if (255 >> 19 !== 0) { 
  throw new Test262Error('#617: 255 >> 19 === 0. Actual: ' + (255 >> 19)); 
} 


if (511 >> 19 !== 0) { 
  throw new Test262Error('#618: 511 >> 19 === 0. Actual: ' + (511 >> 19)); 
} 


if (1023 >> 19 !== 0) { 
  throw new Test262Error('#619: 1023 >> 19 === 0. Actual: ' + (1023 >> 19)); 
} 


if (2047 >> 19 !== 0) { 
  throw new Test262Error('#620: 2047 >> 19 === 0. Actual: ' + (2047 >> 19)); 
} 


if (4095 >> 19 !== 0) { 
  throw new Test262Error('#621: 4095 >> 19 === 0. Actual: ' + (4095 >> 19)); 
} 


if (8191 >> 19 !== 0) { 
  throw new Test262Error('#622: 8191 >> 19 === 0. Actual: ' + (8191 >> 19)); 
} 


if (16383 >> 19 !== 0) { 
  throw new Test262Error('#623: 16383 >> 19 === 0. Actual: ' + (16383 >> 19)); 
} 


if (32767 >> 19 !== 0) { 
  throw new Test262Error('#624: 32767 >> 19 === 0. Actual: ' + (32767 >> 19)); 
} 


if (65535 >> 19 !== 0) { 
  throw new Test262Error('#625: 65535 >> 19 === 0. Actual: ' + (65535 >> 19)); 
} 


if (131071 >> 19 !== 0) { 
  throw new Test262Error('#626: 131071 >> 19 === 0. Actual: ' + (131071 >> 19)); 
} 


if (262143 >> 19 !== 0) { 
  throw new Test262Error('#627: 262143 >> 19 === 0. Actual: ' + (262143 >> 19)); 
} 


if (524287 >> 19 !== 0) { 
  throw new Test262Error('#628: 524287 >> 19 === 0. Actual: ' + (524287 >> 19)); 
} 


if (1048575 >> 19 !== 1) { 
  throw new Test262Error('#629: 1048575 >> 19 === 1. Actual: ' + (1048575 >> 19)); 
} 


if (2097151 >> 19 !== 3) { 
  throw new Test262Error('#630: 2097151 >> 19 === 3. Actual: ' + (2097151 >> 19)); 
} 


if (4194303 >> 19 !== 7) { 
  throw new Test262Error('#631: 4194303 >> 19 === 7. Actual: ' + (4194303 >> 19)); 
} 


if (8388607 >> 19 !== 15) { 
  throw new Test262Error('#632: 8388607 >> 19 === 15. Actual: ' + (8388607 >> 19)); 
} 


if (16777215 >> 19 !== 31) { 
  throw new Test262Error('#633: 16777215 >> 19 === 31. Actual: ' + (16777215 >> 19)); 
} 


if (33554431 >> 19 !== 63) { 
  throw new Test262Error('#634: 33554431 >> 19 === 63. Actual: ' + (33554431 >> 19)); 
} 


if (67108863 >> 19 !== 127) { 
  throw new Test262Error('#635: 67108863 >> 19 === 127. Actual: ' + (67108863 >> 19)); 
} 


if (134217727 >> 19 !== 255) { 
  throw new Test262Error('#636: 134217727 >> 19 === 255. Actual: ' + (134217727 >> 19)); 
} 


if (268435455 >> 19 !== 511) { 
  throw new Test262Error('#637: 268435455 >> 19 === 511. Actual: ' + (268435455 >> 19)); 
} 


if (536870911 >> 19 !== 1023) { 
  throw new Test262Error('#638: 536870911 >> 19 === 1023. Actual: ' + (536870911 >> 19)); 
} 


if (1073741823 >> 19 !== 2047) { 
  throw new Test262Error('#639: 1073741823 >> 19 === 2047. Actual: ' + (1073741823 >> 19)); 
} 


if (2147483647 >> 19 !== 4095) { 
  throw new Test262Error('#640: 2147483647 >> 19 === 4095. Actual: ' + (2147483647 >> 19)); 
} 


if (0 >> 20 !== 0) { 
  throw new Test262Error('#641: 0 >> 20 === 0. Actual: ' + (0 >> 20)); 
} 


if (1 >> 20 !== 0) { 
  throw new Test262Error('#642: 1 >> 20 === 0. Actual: ' + (1 >> 20)); 
} 


if (3 >> 20 !== 0) { 
  throw new Test262Error('#643: 3 >> 20 === 0. Actual: ' + (3 >> 20)); 
} 


if (7 >> 20 !== 0) { 
  throw new Test262Error('#644: 7 >> 20 === 0. Actual: ' + (7 >> 20)); 
} 


if (15 >> 20 !== 0) { 
  throw new Test262Error('#645: 15 >> 20 === 0. Actual: ' + (15 >> 20)); 
} 


if (31 >> 20 !== 0) { 
  throw new Test262Error('#646: 31 >> 20 === 0. Actual: ' + (31 >> 20)); 
} 


if (63 >> 20 !== 0) { 
  throw new Test262Error('#647: 63 >> 20 === 0. Actual: ' + (63 >> 20)); 
} 


if (127 >> 20 !== 0) { 
  throw new Test262Error('#648: 127 >> 20 === 0. Actual: ' + (127 >> 20)); 
} 


if (255 >> 20 !== 0) { 
  throw new Test262Error('#649: 255 >> 20 === 0. Actual: ' + (255 >> 20)); 
} 


if (511 >> 20 !== 0) { 
  throw new Test262Error('#650: 511 >> 20 === 0. Actual: ' + (511 >> 20)); 
} 


if (1023 >> 20 !== 0) { 
  throw new Test262Error('#651: 1023 >> 20 === 0. Actual: ' + (1023 >> 20)); 
} 


if (2047 >> 20 !== 0) { 
  throw new Test262Error('#652: 2047 >> 20 === 0. Actual: ' + (2047 >> 20)); 
} 


if (4095 >> 20 !== 0) { 
  throw new Test262Error('#653: 4095 >> 20 === 0. Actual: ' + (4095 >> 20)); 
} 


if (8191 >> 20 !== 0) { 
  throw new Test262Error('#654: 8191 >> 20 === 0. Actual: ' + (8191 >> 20)); 
} 


if (16383 >> 20 !== 0) { 
  throw new Test262Error('#655: 16383 >> 20 === 0. Actual: ' + (16383 >> 20)); 
} 


if (32767 >> 20 !== 0) { 
  throw new Test262Error('#656: 32767 >> 20 === 0. Actual: ' + (32767 >> 20)); 
} 


if (65535 >> 20 !== 0) { 
  throw new Test262Error('#657: 65535 >> 20 === 0. Actual: ' + (65535 >> 20)); 
} 


if (131071 >> 20 !== 0) { 
  throw new Test262Error('#658: 131071 >> 20 === 0. Actual: ' + (131071 >> 20)); 
} 


if (262143 >> 20 !== 0) { 
  throw new Test262Error('#659: 262143 >> 20 === 0. Actual: ' + (262143 >> 20)); 
} 


if (524287 >> 20 !== 0) { 
  throw new Test262Error('#660: 524287 >> 20 === 0. Actual: ' + (524287 >> 20)); 
} 


if (1048575 >> 20 !== 0) { 
  throw new Test262Error('#661: 1048575 >> 20 === 0. Actual: ' + (1048575 >> 20)); 
} 


if (2097151 >> 20 !== 1) { 
  throw new Test262Error('#662: 2097151 >> 20 === 1. Actual: ' + (2097151 >> 20)); 
} 


if (4194303 >> 20 !== 3) { 
  throw new Test262Error('#663: 4194303 >> 20 === 3. Actual: ' + (4194303 >> 20)); 
} 


if (8388607 >> 20 !== 7) { 
  throw new Test262Error('#664: 8388607 >> 20 === 7. Actual: ' + (8388607 >> 20)); 
} 


if (16777215 >> 20 !== 15) { 
  throw new Test262Error('#665: 16777215 >> 20 === 15. Actual: ' + (16777215 >> 20)); 
} 


if (33554431 >> 20 !== 31) { 
  throw new Test262Error('#666: 33554431 >> 20 === 31. Actual: ' + (33554431 >> 20)); 
} 


if (67108863 >> 20 !== 63) { 
  throw new Test262Error('#667: 67108863 >> 20 === 63. Actual: ' + (67108863 >> 20)); 
} 


if (134217727 >> 20 !== 127) { 
  throw new Test262Error('#668: 134217727 >> 20 === 127. Actual: ' + (134217727 >> 20)); 
} 


if (268435455 >> 20 !== 255) { 
  throw new Test262Error('#669: 268435455 >> 20 === 255. Actual: ' + (268435455 >> 20)); 
} 


if (536870911 >> 20 !== 511) { 
  throw new Test262Error('#670: 536870911 >> 20 === 511. Actual: ' + (536870911 >> 20)); 
} 


if (1073741823 >> 20 !== 1023) { 
  throw new Test262Error('#671: 1073741823 >> 20 === 1023. Actual: ' + (1073741823 >> 20)); 
} 


if (2147483647 >> 20 !== 2047) { 
  throw new Test262Error('#672: 2147483647 >> 20 === 2047. Actual: ' + (2147483647 >> 20)); 
} 


if (0 >> 21 !== 0) { 
  throw new Test262Error('#673: 0 >> 21 === 0. Actual: ' + (0 >> 21)); 
} 


if (1 >> 21 !== 0) { 
  throw new Test262Error('#674: 1 >> 21 === 0. Actual: ' + (1 >> 21)); 
} 


if (3 >> 21 !== 0) { 
  throw new Test262Error('#675: 3 >> 21 === 0. Actual: ' + (3 >> 21)); 
} 


if (7 >> 21 !== 0) { 
  throw new Test262Error('#676: 7 >> 21 === 0. Actual: ' + (7 >> 21)); 
} 


if (15 >> 21 !== 0) { 
  throw new Test262Error('#677: 15 >> 21 === 0. Actual: ' + (15 >> 21)); 
} 


if (31 >> 21 !== 0) { 
  throw new Test262Error('#678: 31 >> 21 === 0. Actual: ' + (31 >> 21)); 
} 


if (63 >> 21 !== 0) { 
  throw new Test262Error('#679: 63 >> 21 === 0. Actual: ' + (63 >> 21)); 
} 


if (127 >> 21 !== 0) { 
  throw new Test262Error('#680: 127 >> 21 === 0. Actual: ' + (127 >> 21)); 
} 


if (255 >> 21 !== 0) { 
  throw new Test262Error('#681: 255 >> 21 === 0. Actual: ' + (255 >> 21)); 
} 


if (511 >> 21 !== 0) { 
  throw new Test262Error('#682: 511 >> 21 === 0. Actual: ' + (511 >> 21)); 
} 


if (1023 >> 21 !== 0) { 
  throw new Test262Error('#683: 1023 >> 21 === 0. Actual: ' + (1023 >> 21)); 
} 


if (2047 >> 21 !== 0) { 
  throw new Test262Error('#684: 2047 >> 21 === 0. Actual: ' + (2047 >> 21)); 
} 


if (4095 >> 21 !== 0) { 
  throw new Test262Error('#685: 4095 >> 21 === 0. Actual: ' + (4095 >> 21)); 
} 


if (8191 >> 21 !== 0) { 
  throw new Test262Error('#686: 8191 >> 21 === 0. Actual: ' + (8191 >> 21)); 
} 


if (16383 >> 21 !== 0) { 
  throw new Test262Error('#687: 16383 >> 21 === 0. Actual: ' + (16383 >> 21)); 
} 


if (32767 >> 21 !== 0) { 
  throw new Test262Error('#688: 32767 >> 21 === 0. Actual: ' + (32767 >> 21)); 
} 


if (65535 >> 21 !== 0) { 
  throw new Test262Error('#689: 65535 >> 21 === 0. Actual: ' + (65535 >> 21)); 
} 


if (131071 >> 21 !== 0) { 
  throw new Test262Error('#690: 131071 >> 21 === 0. Actual: ' + (131071 >> 21)); 
} 


if (262143 >> 21 !== 0) { 
  throw new Test262Error('#691: 262143 >> 21 === 0. Actual: ' + (262143 >> 21)); 
} 


if (524287 >> 21 !== 0) { 
  throw new Test262Error('#692: 524287 >> 21 === 0. Actual: ' + (524287 >> 21)); 
} 


if (1048575 >> 21 !== 0) { 
  throw new Test262Error('#693: 1048575 >> 21 === 0. Actual: ' + (1048575 >> 21)); 
} 


if (2097151 >> 21 !== 0) { 
  throw new Test262Error('#694: 2097151 >> 21 === 0. Actual: ' + (2097151 >> 21)); 
} 


if (4194303 >> 21 !== 1) { 
  throw new Test262Error('#695: 4194303 >> 21 === 1. Actual: ' + (4194303 >> 21)); 
} 


if (8388607 >> 21 !== 3) { 
  throw new Test262Error('#696: 8388607 >> 21 === 3. Actual: ' + (8388607 >> 21)); 
} 


if (16777215 >> 21 !== 7) { 
  throw new Test262Error('#697: 16777215 >> 21 === 7. Actual: ' + (16777215 >> 21)); 
} 


if (33554431 >> 21 !== 15) { 
  throw new Test262Error('#698: 33554431 >> 21 === 15. Actual: ' + (33554431 >> 21)); 
} 


if (67108863 >> 21 !== 31) { 
  throw new Test262Error('#699: 67108863 >> 21 === 31. Actual: ' + (67108863 >> 21)); 
} 


if (134217727 >> 21 !== 63) { 
  throw new Test262Error('#700: 134217727 >> 21 === 63. Actual: ' + (134217727 >> 21)); 
} 


if (268435455 >> 21 !== 127) { 
  throw new Test262Error('#701: 268435455 >> 21 === 127. Actual: ' + (268435455 >> 21)); 
} 


if (536870911 >> 21 !== 255) { 
  throw new Test262Error('#702: 536870911 >> 21 === 255. Actual: ' + (536870911 >> 21)); 
} 


if (1073741823 >> 21 !== 511) { 
  throw new Test262Error('#703: 1073741823 >> 21 === 511. Actual: ' + (1073741823 >> 21)); 
} 


if (2147483647 >> 21 !== 1023) { 
  throw new Test262Error('#704: 2147483647 >> 21 === 1023. Actual: ' + (2147483647 >> 21)); 
} 


if (0 >> 22 !== 0) { 
  throw new Test262Error('#705: 0 >> 22 === 0. Actual: ' + (0 >> 22)); 
} 


if (1 >> 22 !== 0) { 
  throw new Test262Error('#706: 1 >> 22 === 0. Actual: ' + (1 >> 22)); 
} 


if (3 >> 22 !== 0) { 
  throw new Test262Error('#707: 3 >> 22 === 0. Actual: ' + (3 >> 22)); 
} 


if (7 >> 22 !== 0) { 
  throw new Test262Error('#708: 7 >> 22 === 0. Actual: ' + (7 >> 22)); 
} 


if (15 >> 22 !== 0) { 
  throw new Test262Error('#709: 15 >> 22 === 0. Actual: ' + (15 >> 22)); 
} 


if (31 >> 22 !== 0) { 
  throw new Test262Error('#710: 31 >> 22 === 0. Actual: ' + (31 >> 22)); 
} 


if (63 >> 22 !== 0) { 
  throw new Test262Error('#711: 63 >> 22 === 0. Actual: ' + (63 >> 22)); 
} 


if (127 >> 22 !== 0) { 
  throw new Test262Error('#712: 127 >> 22 === 0. Actual: ' + (127 >> 22)); 
} 


if (255 >> 22 !== 0) { 
  throw new Test262Error('#713: 255 >> 22 === 0. Actual: ' + (255 >> 22)); 
} 


if (511 >> 22 !== 0) { 
  throw new Test262Error('#714: 511 >> 22 === 0. Actual: ' + (511 >> 22)); 
} 


if (1023 >> 22 !== 0) { 
  throw new Test262Error('#715: 1023 >> 22 === 0. Actual: ' + (1023 >> 22)); 
} 


if (2047 >> 22 !== 0) { 
  throw new Test262Error('#716: 2047 >> 22 === 0. Actual: ' + (2047 >> 22)); 
} 


if (4095 >> 22 !== 0) { 
  throw new Test262Error('#717: 4095 >> 22 === 0. Actual: ' + (4095 >> 22)); 
} 


if (8191 >> 22 !== 0) { 
  throw new Test262Error('#718: 8191 >> 22 === 0. Actual: ' + (8191 >> 22)); 
} 


if (16383 >> 22 !== 0) { 
  throw new Test262Error('#719: 16383 >> 22 === 0. Actual: ' + (16383 >> 22)); 
} 


if (32767 >> 22 !== 0) { 
  throw new Test262Error('#720: 32767 >> 22 === 0. Actual: ' + (32767 >> 22)); 
} 


if (65535 >> 22 !== 0) { 
  throw new Test262Error('#721: 65535 >> 22 === 0. Actual: ' + (65535 >> 22)); 
} 


if (131071 >> 22 !== 0) { 
  throw new Test262Error('#722: 131071 >> 22 === 0. Actual: ' + (131071 >> 22)); 
} 


if (262143 >> 22 !== 0) { 
  throw new Test262Error('#723: 262143 >> 22 === 0. Actual: ' + (262143 >> 22)); 
} 


if (524287 >> 22 !== 0) { 
  throw new Test262Error('#724: 524287 >> 22 === 0. Actual: ' + (524287 >> 22)); 
} 


if (1048575 >> 22 !== 0) { 
  throw new Test262Error('#725: 1048575 >> 22 === 0. Actual: ' + (1048575 >> 22)); 
} 


if (2097151 >> 22 !== 0) { 
  throw new Test262Error('#726: 2097151 >> 22 === 0. Actual: ' + (2097151 >> 22)); 
} 


if (4194303 >> 22 !== 0) { 
  throw new Test262Error('#727: 4194303 >> 22 === 0. Actual: ' + (4194303 >> 22)); 
} 


if (8388607 >> 22 !== 1) { 
  throw new Test262Error('#728: 8388607 >> 22 === 1. Actual: ' + (8388607 >> 22)); 
} 


if (16777215 >> 22 !== 3) { 
  throw new Test262Error('#729: 16777215 >> 22 === 3. Actual: ' + (16777215 >> 22)); 
} 


if (33554431 >> 22 !== 7) { 
  throw new Test262Error('#730: 33554431 >> 22 === 7. Actual: ' + (33554431 >> 22)); 
} 


if (67108863 >> 22 !== 15) { 
  throw new Test262Error('#731: 67108863 >> 22 === 15. Actual: ' + (67108863 >> 22)); 
} 


if (134217727 >> 22 !== 31) { 
  throw new Test262Error('#732: 134217727 >> 22 === 31. Actual: ' + (134217727 >> 22)); 
} 


if (268435455 >> 22 !== 63) { 
  throw new Test262Error('#733: 268435455 >> 22 === 63. Actual: ' + (268435455 >> 22)); 
} 


if (536870911 >> 22 !== 127) { 
  throw new Test262Error('#734: 536870911 >> 22 === 127. Actual: ' + (536870911 >> 22)); 
} 


if (1073741823 >> 22 !== 255) { 
  throw new Test262Error('#735: 1073741823 >> 22 === 255. Actual: ' + (1073741823 >> 22)); 
} 


if (2147483647 >> 22 !== 511) { 
  throw new Test262Error('#736: 2147483647 >> 22 === 511. Actual: ' + (2147483647 >> 22)); 
} 


if (0 >> 23 !== 0) { 
  throw new Test262Error('#737: 0 >> 23 === 0. Actual: ' + (0 >> 23)); 
} 


if (1 >> 23 !== 0) { 
  throw new Test262Error('#738: 1 >> 23 === 0. Actual: ' + (1 >> 23)); 
} 


if (3 >> 23 !== 0) { 
  throw new Test262Error('#739: 3 >> 23 === 0. Actual: ' + (3 >> 23)); 
} 


if (7 >> 23 !== 0) { 
  throw new Test262Error('#740: 7 >> 23 === 0. Actual: ' + (7 >> 23)); 
} 


if (15 >> 23 !== 0) { 
  throw new Test262Error('#741: 15 >> 23 === 0. Actual: ' + (15 >> 23)); 
} 


if (31 >> 23 !== 0) { 
  throw new Test262Error('#742: 31 >> 23 === 0. Actual: ' + (31 >> 23)); 
} 


if (63 >> 23 !== 0) { 
  throw new Test262Error('#743: 63 >> 23 === 0. Actual: ' + (63 >> 23)); 
} 


if (127 >> 23 !== 0) { 
  throw new Test262Error('#744: 127 >> 23 === 0. Actual: ' + (127 >> 23)); 
} 


if (255 >> 23 !== 0) { 
  throw new Test262Error('#745: 255 >> 23 === 0. Actual: ' + (255 >> 23)); 
} 


if (511 >> 23 !== 0) { 
  throw new Test262Error('#746: 511 >> 23 === 0. Actual: ' + (511 >> 23)); 
} 


if (1023 >> 23 !== 0) { 
  throw new Test262Error('#747: 1023 >> 23 === 0. Actual: ' + (1023 >> 23)); 
} 


if (2047 >> 23 !== 0) { 
  throw new Test262Error('#748: 2047 >> 23 === 0. Actual: ' + (2047 >> 23)); 
} 


if (4095 >> 23 !== 0) { 
  throw new Test262Error('#749: 4095 >> 23 === 0. Actual: ' + (4095 >> 23)); 
} 


if (8191 >> 23 !== 0) { 
  throw new Test262Error('#750: 8191 >> 23 === 0. Actual: ' + (8191 >> 23)); 
} 


if (16383 >> 23 !== 0) { 
  throw new Test262Error('#751: 16383 >> 23 === 0. Actual: ' + (16383 >> 23)); 
} 


if (32767 >> 23 !== 0) { 
  throw new Test262Error('#752: 32767 >> 23 === 0. Actual: ' + (32767 >> 23)); 
} 


if (65535 >> 23 !== 0) { 
  throw new Test262Error('#753: 65535 >> 23 === 0. Actual: ' + (65535 >> 23)); 
} 


if (131071 >> 23 !== 0) { 
  throw new Test262Error('#754: 131071 >> 23 === 0. Actual: ' + (131071 >> 23)); 
} 


if (262143 >> 23 !== 0) { 
  throw new Test262Error('#755: 262143 >> 23 === 0. Actual: ' + (262143 >> 23)); 
} 


if (524287 >> 23 !== 0) { 
  throw new Test262Error('#756: 524287 >> 23 === 0. Actual: ' + (524287 >> 23)); 
} 


if (1048575 >> 23 !== 0) { 
  throw new Test262Error('#757: 1048575 >> 23 === 0. Actual: ' + (1048575 >> 23)); 
} 


if (2097151 >> 23 !== 0) { 
  throw new Test262Error('#758: 2097151 >> 23 === 0. Actual: ' + (2097151 >> 23)); 
} 


if (4194303 >> 23 !== 0) { 
  throw new Test262Error('#759: 4194303 >> 23 === 0. Actual: ' + (4194303 >> 23)); 
} 


if (8388607 >> 23 !== 0) { 
  throw new Test262Error('#760: 8388607 >> 23 === 0. Actual: ' + (8388607 >> 23)); 
} 


if (16777215 >> 23 !== 1) { 
  throw new Test262Error('#761: 16777215 >> 23 === 1. Actual: ' + (16777215 >> 23)); 
} 


if (33554431 >> 23 !== 3) { 
  throw new Test262Error('#762: 33554431 >> 23 === 3. Actual: ' + (33554431 >> 23)); 
} 


if (67108863 >> 23 !== 7) { 
  throw new Test262Error('#763: 67108863 >> 23 === 7. Actual: ' + (67108863 >> 23)); 
} 


if (134217727 >> 23 !== 15) { 
  throw new Test262Error('#764: 134217727 >> 23 === 15. Actual: ' + (134217727 >> 23)); 
} 


if (268435455 >> 23 !== 31) { 
  throw new Test262Error('#765: 268435455 >> 23 === 31. Actual: ' + (268435455 >> 23)); 
} 


if (536870911 >> 23 !== 63) { 
  throw new Test262Error('#766: 536870911 >> 23 === 63. Actual: ' + (536870911 >> 23)); 
} 


if (1073741823 >> 23 !== 127) { 
  throw new Test262Error('#767: 1073741823 >> 23 === 127. Actual: ' + (1073741823 >> 23)); 
} 


if (2147483647 >> 23 !== 255) { 
  throw new Test262Error('#768: 2147483647 >> 23 === 255. Actual: ' + (2147483647 >> 23)); 
} 


if (0 >> 24 !== 0) { 
  throw new Test262Error('#769: 0 >> 24 === 0. Actual: ' + (0 >> 24)); 
} 


if (1 >> 24 !== 0) { 
  throw new Test262Error('#770: 1 >> 24 === 0. Actual: ' + (1 >> 24)); 
} 


if (3 >> 24 !== 0) { 
  throw new Test262Error('#771: 3 >> 24 === 0. Actual: ' + (3 >> 24)); 
} 


if (7 >> 24 !== 0) { 
  throw new Test262Error('#772: 7 >> 24 === 0. Actual: ' + (7 >> 24)); 
} 


if (15 >> 24 !== 0) { 
  throw new Test262Error('#773: 15 >> 24 === 0. Actual: ' + (15 >> 24)); 
} 


if (31 >> 24 !== 0) { 
  throw new Test262Error('#774: 31 >> 24 === 0. Actual: ' + (31 >> 24)); 
} 


if (63 >> 24 !== 0) { 
  throw new Test262Error('#775: 63 >> 24 === 0. Actual: ' + (63 >> 24)); 
} 


if (127 >> 24 !== 0) { 
  throw new Test262Error('#776: 127 >> 24 === 0. Actual: ' + (127 >> 24)); 
} 


if (255 >> 24 !== 0) { 
  throw new Test262Error('#777: 255 >> 24 === 0. Actual: ' + (255 >> 24)); 
} 


if (511 >> 24 !== 0) { 
  throw new Test262Error('#778: 511 >> 24 === 0. Actual: ' + (511 >> 24)); 
} 


if (1023 >> 24 !== 0) { 
  throw new Test262Error('#779: 1023 >> 24 === 0. Actual: ' + (1023 >> 24)); 
} 


if (2047 >> 24 !== 0) { 
  throw new Test262Error('#780: 2047 >> 24 === 0. Actual: ' + (2047 >> 24)); 
} 


if (4095 >> 24 !== 0) { 
  throw new Test262Error('#781: 4095 >> 24 === 0. Actual: ' + (4095 >> 24)); 
} 


if (8191 >> 24 !== 0) { 
  throw new Test262Error('#782: 8191 >> 24 === 0. Actual: ' + (8191 >> 24)); 
} 


if (16383 >> 24 !== 0) { 
  throw new Test262Error('#783: 16383 >> 24 === 0. Actual: ' + (16383 >> 24)); 
} 


if (32767 >> 24 !== 0) { 
  throw new Test262Error('#784: 32767 >> 24 === 0. Actual: ' + (32767 >> 24)); 
} 


if (65535 >> 24 !== 0) { 
  throw new Test262Error('#785: 65535 >> 24 === 0. Actual: ' + (65535 >> 24)); 
} 


if (131071 >> 24 !== 0) { 
  throw new Test262Error('#786: 131071 >> 24 === 0. Actual: ' + (131071 >> 24)); 
} 


if (262143 >> 24 !== 0) { 
  throw new Test262Error('#787: 262143 >> 24 === 0. Actual: ' + (262143 >> 24)); 
} 


if (524287 >> 24 !== 0) { 
  throw new Test262Error('#788: 524287 >> 24 === 0. Actual: ' + (524287 >> 24)); 
} 


if (1048575 >> 24 !== 0) { 
  throw new Test262Error('#789: 1048575 >> 24 === 0. Actual: ' + (1048575 >> 24)); 
} 


if (2097151 >> 24 !== 0) { 
  throw new Test262Error('#790: 2097151 >> 24 === 0. Actual: ' + (2097151 >> 24)); 
} 


if (4194303 >> 24 !== 0) { 
  throw new Test262Error('#791: 4194303 >> 24 === 0. Actual: ' + (4194303 >> 24)); 
} 


if (8388607 >> 24 !== 0) { 
  throw new Test262Error('#792: 8388607 >> 24 === 0. Actual: ' + (8388607 >> 24)); 
} 


if (16777215 >> 24 !== 0) { 
  throw new Test262Error('#793: 16777215 >> 24 === 0. Actual: ' + (16777215 >> 24)); 
} 


if (33554431 >> 24 !== 1) { 
  throw new Test262Error('#794: 33554431 >> 24 === 1. Actual: ' + (33554431 >> 24)); 
} 


if (67108863 >> 24 !== 3) { 
  throw new Test262Error('#795: 67108863 >> 24 === 3. Actual: ' + (67108863 >> 24)); 
} 


if (134217727 >> 24 !== 7) { 
  throw new Test262Error('#796: 134217727 >> 24 === 7. Actual: ' + (134217727 >> 24)); 
} 


if (268435455 >> 24 !== 15) { 
  throw new Test262Error('#797: 268435455 >> 24 === 15. Actual: ' + (268435455 >> 24)); 
} 


if (536870911 >> 24 !== 31) { 
  throw new Test262Error('#798: 536870911 >> 24 === 31. Actual: ' + (536870911 >> 24)); 
} 


if (1073741823 >> 24 !== 63) { 
  throw new Test262Error('#799: 1073741823 >> 24 === 63. Actual: ' + (1073741823 >> 24)); 
} 


if (2147483647 >> 24 !== 127) { 
  throw new Test262Error('#800: 2147483647 >> 24 === 127. Actual: ' + (2147483647 >> 24)); 
} 


if (0 >> 25 !== 0) { 
  throw new Test262Error('#801: 0 >> 25 === 0. Actual: ' + (0 >> 25)); 
} 


if (1 >> 25 !== 0) { 
  throw new Test262Error('#802: 1 >> 25 === 0. Actual: ' + (1 >> 25)); 
} 


if (3 >> 25 !== 0) { 
  throw new Test262Error('#803: 3 >> 25 === 0. Actual: ' + (3 >> 25)); 
} 


if (7 >> 25 !== 0) { 
  throw new Test262Error('#804: 7 >> 25 === 0. Actual: ' + (7 >> 25)); 
} 


if (15 >> 25 !== 0) { 
  throw new Test262Error('#805: 15 >> 25 === 0. Actual: ' + (15 >> 25)); 
} 


if (31 >> 25 !== 0) { 
  throw new Test262Error('#806: 31 >> 25 === 0. Actual: ' + (31 >> 25)); 
} 


if (63 >> 25 !== 0) { 
  throw new Test262Error('#807: 63 >> 25 === 0. Actual: ' + (63 >> 25)); 
} 


if (127 >> 25 !== 0) { 
  throw new Test262Error('#808: 127 >> 25 === 0. Actual: ' + (127 >> 25)); 
} 


if (255 >> 25 !== 0) { 
  throw new Test262Error('#809: 255 >> 25 === 0. Actual: ' + (255 >> 25)); 
} 


if (511 >> 25 !== 0) { 
  throw new Test262Error('#810: 511 >> 25 === 0. Actual: ' + (511 >> 25)); 
} 


if (1023 >> 25 !== 0) { 
  throw new Test262Error('#811: 1023 >> 25 === 0. Actual: ' + (1023 >> 25)); 
} 


if (2047 >> 25 !== 0) { 
  throw new Test262Error('#812: 2047 >> 25 === 0. Actual: ' + (2047 >> 25)); 
} 


if (4095 >> 25 !== 0) { 
  throw new Test262Error('#813: 4095 >> 25 === 0. Actual: ' + (4095 >> 25)); 
} 


if (8191 >> 25 !== 0) { 
  throw new Test262Error('#814: 8191 >> 25 === 0. Actual: ' + (8191 >> 25)); 
} 


if (16383 >> 25 !== 0) { 
  throw new Test262Error('#815: 16383 >> 25 === 0. Actual: ' + (16383 >> 25)); 
} 


if (32767 >> 25 !== 0) { 
  throw new Test262Error('#816: 32767 >> 25 === 0. Actual: ' + (32767 >> 25)); 
} 


if (65535 >> 25 !== 0) { 
  throw new Test262Error('#817: 65535 >> 25 === 0. Actual: ' + (65535 >> 25)); 
} 


if (131071 >> 25 !== 0) { 
  throw new Test262Error('#818: 131071 >> 25 === 0. Actual: ' + (131071 >> 25)); 
} 


if (262143 >> 25 !== 0) { 
  throw new Test262Error('#819: 262143 >> 25 === 0. Actual: ' + (262143 >> 25)); 
} 


if (524287 >> 25 !== 0) { 
  throw new Test262Error('#820: 524287 >> 25 === 0. Actual: ' + (524287 >> 25)); 
} 


if (1048575 >> 25 !== 0) { 
  throw new Test262Error('#821: 1048575 >> 25 === 0. Actual: ' + (1048575 >> 25)); 
} 


if (2097151 >> 25 !== 0) { 
  throw new Test262Error('#822: 2097151 >> 25 === 0. Actual: ' + (2097151 >> 25)); 
} 


if (4194303 >> 25 !== 0) { 
  throw new Test262Error('#823: 4194303 >> 25 === 0. Actual: ' + (4194303 >> 25)); 
} 


if (8388607 >> 25 !== 0) { 
  throw new Test262Error('#824: 8388607 >> 25 === 0. Actual: ' + (8388607 >> 25)); 
} 


if (16777215 >> 25 !== 0) { 
  throw new Test262Error('#825: 16777215 >> 25 === 0. Actual: ' + (16777215 >> 25)); 
} 


if (33554431 >> 25 !== 0) { 
  throw new Test262Error('#826: 33554431 >> 25 === 0. Actual: ' + (33554431 >> 25)); 
} 


if (67108863 >> 25 !== 1) { 
  throw new Test262Error('#827: 67108863 >> 25 === 1. Actual: ' + (67108863 >> 25)); 
} 


if (134217727 >> 25 !== 3) { 
  throw new Test262Error('#828: 134217727 >> 25 === 3. Actual: ' + (134217727 >> 25)); 
} 


if (268435455 >> 25 !== 7) { 
  throw new Test262Error('#829: 268435455 >> 25 === 7. Actual: ' + (268435455 >> 25)); 
} 


if (536870911 >> 25 !== 15) { 
  throw new Test262Error('#830: 536870911 >> 25 === 15. Actual: ' + (536870911 >> 25)); 
} 


if (1073741823 >> 25 !== 31) { 
  throw new Test262Error('#831: 1073741823 >> 25 === 31. Actual: ' + (1073741823 >> 25)); 
} 


if (2147483647 >> 25 !== 63) { 
  throw new Test262Error('#832: 2147483647 >> 25 === 63. Actual: ' + (2147483647 >> 25)); 
} 


if (0 >> 26 !== 0) { 
  throw new Test262Error('#833: 0 >> 26 === 0. Actual: ' + (0 >> 26)); 
} 


if (1 >> 26 !== 0) { 
  throw new Test262Error('#834: 1 >> 26 === 0. Actual: ' + (1 >> 26)); 
} 


if (3 >> 26 !== 0) { 
  throw new Test262Error('#835: 3 >> 26 === 0. Actual: ' + (3 >> 26)); 
} 


if (7 >> 26 !== 0) { 
  throw new Test262Error('#836: 7 >> 26 === 0. Actual: ' + (7 >> 26)); 
} 


if (15 >> 26 !== 0) { 
  throw new Test262Error('#837: 15 >> 26 === 0. Actual: ' + (15 >> 26)); 
} 


if (31 >> 26 !== 0) { 
  throw new Test262Error('#838: 31 >> 26 === 0. Actual: ' + (31 >> 26)); 
} 


if (63 >> 26 !== 0) { 
  throw new Test262Error('#839: 63 >> 26 === 0. Actual: ' + (63 >> 26)); 
} 


if (127 >> 26 !== 0) { 
  throw new Test262Error('#840: 127 >> 26 === 0. Actual: ' + (127 >> 26)); 
} 


if (255 >> 26 !== 0) { 
  throw new Test262Error('#841: 255 >> 26 === 0. Actual: ' + (255 >> 26)); 
} 


if (511 >> 26 !== 0) { 
  throw new Test262Error('#842: 511 >> 26 === 0. Actual: ' + (511 >> 26)); 
} 


if (1023 >> 26 !== 0) { 
  throw new Test262Error('#843: 1023 >> 26 === 0. Actual: ' + (1023 >> 26)); 
} 


if (2047 >> 26 !== 0) { 
  throw new Test262Error('#844: 2047 >> 26 === 0. Actual: ' + (2047 >> 26)); 
} 


if (4095 >> 26 !== 0) { 
  throw new Test262Error('#845: 4095 >> 26 === 0. Actual: ' + (4095 >> 26)); 
} 


if (8191 >> 26 !== 0) { 
  throw new Test262Error('#846: 8191 >> 26 === 0. Actual: ' + (8191 >> 26)); 
} 


if (16383 >> 26 !== 0) { 
  throw new Test262Error('#847: 16383 >> 26 === 0. Actual: ' + (16383 >> 26)); 
} 


if (32767 >> 26 !== 0) { 
  throw new Test262Error('#848: 32767 >> 26 === 0. Actual: ' + (32767 >> 26)); 
} 


if (65535 >> 26 !== 0) { 
  throw new Test262Error('#849: 65535 >> 26 === 0. Actual: ' + (65535 >> 26)); 
} 


if (131071 >> 26 !== 0) { 
  throw new Test262Error('#850: 131071 >> 26 === 0. Actual: ' + (131071 >> 26)); 
} 


if (262143 >> 26 !== 0) { 
  throw new Test262Error('#851: 262143 >> 26 === 0. Actual: ' + (262143 >> 26)); 
} 


if (524287 >> 26 !== 0) { 
  throw new Test262Error('#852: 524287 >> 26 === 0. Actual: ' + (524287 >> 26)); 
} 


if (1048575 >> 26 !== 0) { 
  throw new Test262Error('#853: 1048575 >> 26 === 0. Actual: ' + (1048575 >> 26)); 
} 


if (2097151 >> 26 !== 0) { 
  throw new Test262Error('#854: 2097151 >> 26 === 0. Actual: ' + (2097151 >> 26)); 
} 


if (4194303 >> 26 !== 0) { 
  throw new Test262Error('#855: 4194303 >> 26 === 0. Actual: ' + (4194303 >> 26)); 
} 


if (8388607 >> 26 !== 0) { 
  throw new Test262Error('#856: 8388607 >> 26 === 0. Actual: ' + (8388607 >> 26)); 
} 


if (16777215 >> 26 !== 0) { 
  throw new Test262Error('#857: 16777215 >> 26 === 0. Actual: ' + (16777215 >> 26)); 
} 


if (33554431 >> 26 !== 0) { 
  throw new Test262Error('#858: 33554431 >> 26 === 0. Actual: ' + (33554431 >> 26)); 
} 


if (67108863 >> 26 !== 0) { 
  throw new Test262Error('#859: 67108863 >> 26 === 0. Actual: ' + (67108863 >> 26)); 
} 


if (134217727 >> 26 !== 1) { 
  throw new Test262Error('#860: 134217727 >> 26 === 1. Actual: ' + (134217727 >> 26)); 
} 


if (268435455 >> 26 !== 3) { 
  throw new Test262Error('#861: 268435455 >> 26 === 3. Actual: ' + (268435455 >> 26)); 
} 


if (536870911 >> 26 !== 7) { 
  throw new Test262Error('#862: 536870911 >> 26 === 7. Actual: ' + (536870911 >> 26)); 
} 


if (1073741823 >> 26 !== 15) { 
  throw new Test262Error('#863: 1073741823 >> 26 === 15. Actual: ' + (1073741823 >> 26)); 
} 


if (2147483647 >> 26 !== 31) { 
  throw new Test262Error('#864: 2147483647 >> 26 === 31. Actual: ' + (2147483647 >> 26)); 
} 


if (0 >> 27 !== 0) { 
  throw new Test262Error('#865: 0 >> 27 === 0. Actual: ' + (0 >> 27)); 
} 


if (1 >> 27 !== 0) { 
  throw new Test262Error('#866: 1 >> 27 === 0. Actual: ' + (1 >> 27)); 
} 


if (3 >> 27 !== 0) { 
  throw new Test262Error('#867: 3 >> 27 === 0. Actual: ' + (3 >> 27)); 
} 


if (7 >> 27 !== 0) { 
  throw new Test262Error('#868: 7 >> 27 === 0. Actual: ' + (7 >> 27)); 
} 


if (15 >> 27 !== 0) { 
  throw new Test262Error('#869: 15 >> 27 === 0. Actual: ' + (15 >> 27)); 
} 


if (31 >> 27 !== 0) { 
  throw new Test262Error('#870: 31 >> 27 === 0. Actual: ' + (31 >> 27)); 
} 


if (63 >> 27 !== 0) { 
  throw new Test262Error('#871: 63 >> 27 === 0. Actual: ' + (63 >> 27)); 
} 


if (127 >> 27 !== 0) { 
  throw new Test262Error('#872: 127 >> 27 === 0. Actual: ' + (127 >> 27)); 
} 


if (255 >> 27 !== 0) { 
  throw new Test262Error('#873: 255 >> 27 === 0. Actual: ' + (255 >> 27)); 
} 


if (511 >> 27 !== 0) { 
  throw new Test262Error('#874: 511 >> 27 === 0. Actual: ' + (511 >> 27)); 
} 


if (1023 >> 27 !== 0) { 
  throw new Test262Error('#875: 1023 >> 27 === 0. Actual: ' + (1023 >> 27)); 
} 


if (2047 >> 27 !== 0) { 
  throw new Test262Error('#876: 2047 >> 27 === 0. Actual: ' + (2047 >> 27)); 
} 


if (4095 >> 27 !== 0) { 
  throw new Test262Error('#877: 4095 >> 27 === 0. Actual: ' + (4095 >> 27)); 
} 


if (8191 >> 27 !== 0) { 
  throw new Test262Error('#878: 8191 >> 27 === 0. Actual: ' + (8191 >> 27)); 
} 


if (16383 >> 27 !== 0) { 
  throw new Test262Error('#879: 16383 >> 27 === 0. Actual: ' + (16383 >> 27)); 
} 


if (32767 >> 27 !== 0) { 
  throw new Test262Error('#880: 32767 >> 27 === 0. Actual: ' + (32767 >> 27)); 
} 


if (65535 >> 27 !== 0) { 
  throw new Test262Error('#881: 65535 >> 27 === 0. Actual: ' + (65535 >> 27)); 
} 


if (131071 >> 27 !== 0) { 
  throw new Test262Error('#882: 131071 >> 27 === 0. Actual: ' + (131071 >> 27)); 
} 


if (262143 >> 27 !== 0) { 
  throw new Test262Error('#883: 262143 >> 27 === 0. Actual: ' + (262143 >> 27)); 
} 


if (524287 >> 27 !== 0) { 
  throw new Test262Error('#884: 524287 >> 27 === 0. Actual: ' + (524287 >> 27)); 
} 


if (1048575 >> 27 !== 0) { 
  throw new Test262Error('#885: 1048575 >> 27 === 0. Actual: ' + (1048575 >> 27)); 
} 


if (2097151 >> 27 !== 0) { 
  throw new Test262Error('#886: 2097151 >> 27 === 0. Actual: ' + (2097151 >> 27)); 
} 


if (4194303 >> 27 !== 0) { 
  throw new Test262Error('#887: 4194303 >> 27 === 0. Actual: ' + (4194303 >> 27)); 
} 


if (8388607 >> 27 !== 0) { 
  throw new Test262Error('#888: 8388607 >> 27 === 0. Actual: ' + (8388607 >> 27)); 
} 


if (16777215 >> 27 !== 0) { 
  throw new Test262Error('#889: 16777215 >> 27 === 0. Actual: ' + (16777215 >> 27)); 
} 


if (33554431 >> 27 !== 0) { 
  throw new Test262Error('#890: 33554431 >> 27 === 0. Actual: ' + (33554431 >> 27)); 
} 


if (67108863 >> 27 !== 0) { 
  throw new Test262Error('#891: 67108863 >> 27 === 0. Actual: ' + (67108863 >> 27)); 
} 


if (134217727 >> 27 !== 0) { 
  throw new Test262Error('#892: 134217727 >> 27 === 0. Actual: ' + (134217727 >> 27)); 
} 


if (268435455 >> 27 !== 1) { 
  throw new Test262Error('#893: 268435455 >> 27 === 1. Actual: ' + (268435455 >> 27)); 
} 


if (536870911 >> 27 !== 3) { 
  throw new Test262Error('#894: 536870911 >> 27 === 3. Actual: ' + (536870911 >> 27)); 
} 


if (1073741823 >> 27 !== 7) { 
  throw new Test262Error('#895: 1073741823 >> 27 === 7. Actual: ' + (1073741823 >> 27)); 
} 


if (2147483647 >> 27 !== 15) { 
  throw new Test262Error('#896: 2147483647 >> 27 === 15. Actual: ' + (2147483647 >> 27)); 
} 


if (0 >> 28 !== 0) { 
  throw new Test262Error('#897: 0 >> 28 === 0. Actual: ' + (0 >> 28)); 
} 


if (1 >> 28 !== 0) { 
  throw new Test262Error('#898: 1 >> 28 === 0. Actual: ' + (1 >> 28)); 
} 


if (3 >> 28 !== 0) { 
  throw new Test262Error('#899: 3 >> 28 === 0. Actual: ' + (3 >> 28)); 
} 


if (7 >> 28 !== 0) { 
  throw new Test262Error('#900: 7 >> 28 === 0. Actual: ' + (7 >> 28)); 
} 


if (15 >> 28 !== 0) { 
  throw new Test262Error('#901: 15 >> 28 === 0. Actual: ' + (15 >> 28)); 
} 


if (31 >> 28 !== 0) { 
  throw new Test262Error('#902: 31 >> 28 === 0. Actual: ' + (31 >> 28)); 
} 


if (63 >> 28 !== 0) { 
  throw new Test262Error('#903: 63 >> 28 === 0. Actual: ' + (63 >> 28)); 
} 


if (127 >> 28 !== 0) { 
  throw new Test262Error('#904: 127 >> 28 === 0. Actual: ' + (127 >> 28)); 
} 


if (255 >> 28 !== 0) { 
  throw new Test262Error('#905: 255 >> 28 === 0. Actual: ' + (255 >> 28)); 
} 


if (511 >> 28 !== 0) { 
  throw new Test262Error('#906: 511 >> 28 === 0. Actual: ' + (511 >> 28)); 
} 


if (1023 >> 28 !== 0) { 
  throw new Test262Error('#907: 1023 >> 28 === 0. Actual: ' + (1023 >> 28)); 
} 


if (2047 >> 28 !== 0) { 
  throw new Test262Error('#908: 2047 >> 28 === 0. Actual: ' + (2047 >> 28)); 
} 


if (4095 >> 28 !== 0) { 
  throw new Test262Error('#909: 4095 >> 28 === 0. Actual: ' + (4095 >> 28)); 
} 


if (8191 >> 28 !== 0) { 
  throw new Test262Error('#910: 8191 >> 28 === 0. Actual: ' + (8191 >> 28)); 
} 


if (16383 >> 28 !== 0) { 
  throw new Test262Error('#911: 16383 >> 28 === 0. Actual: ' + (16383 >> 28)); 
} 


if (32767 >> 28 !== 0) { 
  throw new Test262Error('#912: 32767 >> 28 === 0. Actual: ' + (32767 >> 28)); 
} 


if (65535 >> 28 !== 0) { 
  throw new Test262Error('#913: 65535 >> 28 === 0. Actual: ' + (65535 >> 28)); 
} 


if (131071 >> 28 !== 0) { 
  throw new Test262Error('#914: 131071 >> 28 === 0. Actual: ' + (131071 >> 28)); 
} 


if (262143 >> 28 !== 0) { 
  throw new Test262Error('#915: 262143 >> 28 === 0. Actual: ' + (262143 >> 28)); 
} 


if (524287 >> 28 !== 0) { 
  throw new Test262Error('#916: 524287 >> 28 === 0. Actual: ' + (524287 >> 28)); 
} 


if (1048575 >> 28 !== 0) { 
  throw new Test262Error('#917: 1048575 >> 28 === 0. Actual: ' + (1048575 >> 28)); 
} 


if (2097151 >> 28 !== 0) { 
  throw new Test262Error('#918: 2097151 >> 28 === 0. Actual: ' + (2097151 >> 28)); 
} 


if (4194303 >> 28 !== 0) { 
  throw new Test262Error('#919: 4194303 >> 28 === 0. Actual: ' + (4194303 >> 28)); 
} 


if (8388607 >> 28 !== 0) { 
  throw new Test262Error('#920: 8388607 >> 28 === 0. Actual: ' + (8388607 >> 28)); 
} 


if (16777215 >> 28 !== 0) { 
  throw new Test262Error('#921: 16777215 >> 28 === 0. Actual: ' + (16777215 >> 28)); 
} 


if (33554431 >> 28 !== 0) { 
  throw new Test262Error('#922: 33554431 >> 28 === 0. Actual: ' + (33554431 >> 28)); 
} 


if (67108863 >> 28 !== 0) { 
  throw new Test262Error('#923: 67108863 >> 28 === 0. Actual: ' + (67108863 >> 28)); 
} 


if (134217727 >> 28 !== 0) { 
  throw new Test262Error('#924: 134217727 >> 28 === 0. Actual: ' + (134217727 >> 28)); 
} 


if (268435455 >> 28 !== 0) { 
  throw new Test262Error('#925: 268435455 >> 28 === 0. Actual: ' + (268435455 >> 28)); 
} 


if (536870911 >> 28 !== 1) { 
  throw new Test262Error('#926: 536870911 >> 28 === 1. Actual: ' + (536870911 >> 28)); 
} 


if (1073741823 >> 28 !== 3) { 
  throw new Test262Error('#927: 1073741823 >> 28 === 3. Actual: ' + (1073741823 >> 28)); 
} 


if (2147483647 >> 28 !== 7) { 
  throw new Test262Error('#928: 2147483647 >> 28 === 7. Actual: ' + (2147483647 >> 28)); 
} 


if (0 >> 29 !== 0) { 
  throw new Test262Error('#929: 0 >> 29 === 0. Actual: ' + (0 >> 29)); 
} 


if (1 >> 29 !== 0) { 
  throw new Test262Error('#930: 1 >> 29 === 0. Actual: ' + (1 >> 29)); 
} 


if (3 >> 29 !== 0) { 
  throw new Test262Error('#931: 3 >> 29 === 0. Actual: ' + (3 >> 29)); 
} 


if (7 >> 29 !== 0) { 
  throw new Test262Error('#932: 7 >> 29 === 0. Actual: ' + (7 >> 29)); 
} 


if (15 >> 29 !== 0) { 
  throw new Test262Error('#933: 15 >> 29 === 0. Actual: ' + (15 >> 29)); 
} 


if (31 >> 29 !== 0) { 
  throw new Test262Error('#934: 31 >> 29 === 0. Actual: ' + (31 >> 29)); 
} 


if (63 >> 29 !== 0) { 
  throw new Test262Error('#935: 63 >> 29 === 0. Actual: ' + (63 >> 29)); 
} 


if (127 >> 29 !== 0) { 
  throw new Test262Error('#936: 127 >> 29 === 0. Actual: ' + (127 >> 29)); 
} 


if (255 >> 29 !== 0) { 
  throw new Test262Error('#937: 255 >> 29 === 0. Actual: ' + (255 >> 29)); 
} 


if (511 >> 29 !== 0) { 
  throw new Test262Error('#938: 511 >> 29 === 0. Actual: ' + (511 >> 29)); 
} 


if (1023 >> 29 !== 0) { 
  throw new Test262Error('#939: 1023 >> 29 === 0. Actual: ' + (1023 >> 29)); 
} 


if (2047 >> 29 !== 0) { 
  throw new Test262Error('#940: 2047 >> 29 === 0. Actual: ' + (2047 >> 29)); 
} 


if (4095 >> 29 !== 0) { 
  throw new Test262Error('#941: 4095 >> 29 === 0. Actual: ' + (4095 >> 29)); 
} 


if (8191 >> 29 !== 0) { 
  throw new Test262Error('#942: 8191 >> 29 === 0. Actual: ' + (8191 >> 29)); 
} 


if (16383 >> 29 !== 0) { 
  throw new Test262Error('#943: 16383 >> 29 === 0. Actual: ' + (16383 >> 29)); 
} 


if (32767 >> 29 !== 0) { 
  throw new Test262Error('#944: 32767 >> 29 === 0. Actual: ' + (32767 >> 29)); 
} 


if (65535 >> 29 !== 0) { 
  throw new Test262Error('#945: 65535 >> 29 === 0. Actual: ' + (65535 >> 29)); 
} 


if (131071 >> 29 !== 0) { 
  throw new Test262Error('#946: 131071 >> 29 === 0. Actual: ' + (131071 >> 29)); 
} 


if (262143 >> 29 !== 0) { 
  throw new Test262Error('#947: 262143 >> 29 === 0. Actual: ' + (262143 >> 29)); 
} 


if (524287 >> 29 !== 0) { 
  throw new Test262Error('#948: 524287 >> 29 === 0. Actual: ' + (524287 >> 29)); 
} 


if (1048575 >> 29 !== 0) { 
  throw new Test262Error('#949: 1048575 >> 29 === 0. Actual: ' + (1048575 >> 29)); 
} 


if (2097151 >> 29 !== 0) { 
  throw new Test262Error('#950: 2097151 >> 29 === 0. Actual: ' + (2097151 >> 29)); 
} 


if (4194303 >> 29 !== 0) { 
  throw new Test262Error('#951: 4194303 >> 29 === 0. Actual: ' + (4194303 >> 29)); 
} 


if (8388607 >> 29 !== 0) { 
  throw new Test262Error('#952: 8388607 >> 29 === 0. Actual: ' + (8388607 >> 29)); 
} 


if (16777215 >> 29 !== 0) { 
  throw new Test262Error('#953: 16777215 >> 29 === 0. Actual: ' + (16777215 >> 29)); 
} 


if (33554431 >> 29 !== 0) { 
  throw new Test262Error('#954: 33554431 >> 29 === 0. Actual: ' + (33554431 >> 29)); 
} 


if (67108863 >> 29 !== 0) { 
  throw new Test262Error('#955: 67108863 >> 29 === 0. Actual: ' + (67108863 >> 29)); 
} 


if (134217727 >> 29 !== 0) { 
  throw new Test262Error('#956: 134217727 >> 29 === 0. Actual: ' + (134217727 >> 29)); 
} 


if (268435455 >> 29 !== 0) { 
  throw new Test262Error('#957: 268435455 >> 29 === 0. Actual: ' + (268435455 >> 29)); 
} 


if (536870911 >> 29 !== 0) { 
  throw new Test262Error('#958: 536870911 >> 29 === 0. Actual: ' + (536870911 >> 29)); 
} 


if (1073741823 >> 29 !== 1) { 
  throw new Test262Error('#959: 1073741823 >> 29 === 1. Actual: ' + (1073741823 >> 29)); 
} 


if (2147483647 >> 29 !== 3) { 
  throw new Test262Error('#960: 2147483647 >> 29 === 3. Actual: ' + (2147483647 >> 29)); 
} 


if (0 >> 30 !== 0) { 
  throw new Test262Error('#961: 0 >> 30 === 0. Actual: ' + (0 >> 30)); 
} 


if (1 >> 30 !== 0) { 
  throw new Test262Error('#962: 1 >> 30 === 0. Actual: ' + (1 >> 30)); 
} 


if (3 >> 30 !== 0) { 
  throw new Test262Error('#963: 3 >> 30 === 0. Actual: ' + (3 >> 30)); 
} 


if (7 >> 30 !== 0) { 
  throw new Test262Error('#964: 7 >> 30 === 0. Actual: ' + (7 >> 30)); 
} 


if (15 >> 30 !== 0) { 
  throw new Test262Error('#965: 15 >> 30 === 0. Actual: ' + (15 >> 30)); 
} 


if (31 >> 30 !== 0) { 
  throw new Test262Error('#966: 31 >> 30 === 0. Actual: ' + (31 >> 30)); 
} 


if (63 >> 30 !== 0) { 
  throw new Test262Error('#967: 63 >> 30 === 0. Actual: ' + (63 >> 30)); 
} 


if (127 >> 30 !== 0) { 
  throw new Test262Error('#968: 127 >> 30 === 0. Actual: ' + (127 >> 30)); 
} 


if (255 >> 30 !== 0) { 
  throw new Test262Error('#969: 255 >> 30 === 0. Actual: ' + (255 >> 30)); 
} 


if (511 >> 30 !== 0) { 
  throw new Test262Error('#970: 511 >> 30 === 0. Actual: ' + (511 >> 30)); 
} 


if (1023 >> 30 !== 0) { 
  throw new Test262Error('#971: 1023 >> 30 === 0. Actual: ' + (1023 >> 30)); 
} 


if (2047 >> 30 !== 0) { 
  throw new Test262Error('#972: 2047 >> 30 === 0. Actual: ' + (2047 >> 30)); 
} 


if (4095 >> 30 !== 0) { 
  throw new Test262Error('#973: 4095 >> 30 === 0. Actual: ' + (4095 >> 30)); 
} 


if (8191 >> 30 !== 0) { 
  throw new Test262Error('#974: 8191 >> 30 === 0. Actual: ' + (8191 >> 30)); 
} 


if (16383 >> 30 !== 0) { 
  throw new Test262Error('#975: 16383 >> 30 === 0. Actual: ' + (16383 >> 30)); 
} 


if (32767 >> 30 !== 0) { 
  throw new Test262Error('#976: 32767 >> 30 === 0. Actual: ' + (32767 >> 30)); 
} 


if (65535 >> 30 !== 0) { 
  throw new Test262Error('#977: 65535 >> 30 === 0. Actual: ' + (65535 >> 30)); 
} 


if (131071 >> 30 !== 0) { 
  throw new Test262Error('#978: 131071 >> 30 === 0. Actual: ' + (131071 >> 30)); 
} 


if (262143 >> 30 !== 0) { 
  throw new Test262Error('#979: 262143 >> 30 === 0. Actual: ' + (262143 >> 30)); 
} 


if (524287 >> 30 !== 0) { 
  throw new Test262Error('#980: 524287 >> 30 === 0. Actual: ' + (524287 >> 30)); 
} 


if (1048575 >> 30 !== 0) { 
  throw new Test262Error('#981: 1048575 >> 30 === 0. Actual: ' + (1048575 >> 30)); 
} 


if (2097151 >> 30 !== 0) { 
  throw new Test262Error('#982: 2097151 >> 30 === 0. Actual: ' + (2097151 >> 30)); 
} 


if (4194303 >> 30 !== 0) { 
  throw new Test262Error('#983: 4194303 >> 30 === 0. Actual: ' + (4194303 >> 30)); 
} 


if (8388607 >> 30 !== 0) { 
  throw new Test262Error('#984: 8388607 >> 30 === 0. Actual: ' + (8388607 >> 30)); 
} 


if (16777215 >> 30 !== 0) { 
  throw new Test262Error('#985: 16777215 >> 30 === 0. Actual: ' + (16777215 >> 30)); 
} 


if (33554431 >> 30 !== 0) { 
  throw new Test262Error('#986: 33554431 >> 30 === 0. Actual: ' + (33554431 >> 30)); 
} 


if (67108863 >> 30 !== 0) { 
  throw new Test262Error('#987: 67108863 >> 30 === 0. Actual: ' + (67108863 >> 30)); 
} 


if (134217727 >> 30 !== 0) { 
  throw new Test262Error('#988: 134217727 >> 30 === 0. Actual: ' + (134217727 >> 30)); 
} 


if (268435455 >> 30 !== 0) { 
  throw new Test262Error('#989: 268435455 >> 30 === 0. Actual: ' + (268435455 >> 30)); 
} 


if (536870911 >> 30 !== 0) { 
  throw new Test262Error('#990: 536870911 >> 30 === 0. Actual: ' + (536870911 >> 30)); 
} 


if (1073741823 >> 30 !== 0) { 
  throw new Test262Error('#991: 1073741823 >> 30 === 0. Actual: ' + (1073741823 >> 30)); 
} 


if (2147483647 >> 30 !== 1) { 
  throw new Test262Error('#992: 2147483647 >> 30 === 1. Actual: ' + (2147483647 >> 30)); 
} 


if (0 >> 31 !== 0) { 
  throw new Test262Error('#993: 0 >> 31 === 0. Actual: ' + (0 >> 31)); 
} 


if (1 >> 31 !== 0) { 
  throw new Test262Error('#994: 1 >> 31 === 0. Actual: ' + (1 >> 31)); 
} 


if (3 >> 31 !== 0) { 
  throw new Test262Error('#995: 3 >> 31 === 0. Actual: ' + (3 >> 31)); 
} 


if (7 >> 31 !== 0) { 
  throw new Test262Error('#996: 7 >> 31 === 0. Actual: ' + (7 >> 31)); 
} 


if (15 >> 31 !== 0) { 
  throw new Test262Error('#997: 15 >> 31 === 0. Actual: ' + (15 >> 31)); 
} 


if (31 >> 31 !== 0) { 
  throw new Test262Error('#998: 31 >> 31 === 0. Actual: ' + (31 >> 31)); 
} 


if (63 >> 31 !== 0) { 
  throw new Test262Error('#999: 63 >> 31 === 0. Actual: ' + (63 >> 31)); 
} 


if (127 >> 31 !== 0) { 
  throw new Test262Error('#1000: 127 >> 31 === 0. Actual: ' + (127 >> 31)); 
} 


if (255 >> 31 !== 0) { 
  throw new Test262Error('#1001: 255 >> 31 === 0. Actual: ' + (255 >> 31)); 
} 


if (511 >> 31 !== 0) { 
  throw new Test262Error('#1002: 511 >> 31 === 0. Actual: ' + (511 >> 31)); 
} 


if (1023 >> 31 !== 0) { 
  throw new Test262Error('#1003: 1023 >> 31 === 0. Actual: ' + (1023 >> 31)); 
} 


if (2047 >> 31 !== 0) { 
  throw new Test262Error('#1004: 2047 >> 31 === 0. Actual: ' + (2047 >> 31)); 
} 


if (4095 >> 31 !== 0) { 
  throw new Test262Error('#1005: 4095 >> 31 === 0. Actual: ' + (4095 >> 31)); 
} 


if (8191 >> 31 !== 0) { 
  throw new Test262Error('#1006: 8191 >> 31 === 0. Actual: ' + (8191 >> 31)); 
} 


if (16383 >> 31 !== 0) { 
  throw new Test262Error('#1007: 16383 >> 31 === 0. Actual: ' + (16383 >> 31)); 
} 


if (32767 >> 31 !== 0) { 
  throw new Test262Error('#1008: 32767 >> 31 === 0. Actual: ' + (32767 >> 31)); 
} 


if (65535 >> 31 !== 0) { 
  throw new Test262Error('#1009: 65535 >> 31 === 0. Actual: ' + (65535 >> 31)); 
} 


if (131071 >> 31 !== 0) { 
  throw new Test262Error('#1010: 131071 >> 31 === 0. Actual: ' + (131071 >> 31)); 
} 


if (262143 >> 31 !== 0) { 
  throw new Test262Error('#1011: 262143 >> 31 === 0. Actual: ' + (262143 >> 31)); 
} 


if (524287 >> 31 !== 0) { 
  throw new Test262Error('#1012: 524287 >> 31 === 0. Actual: ' + (524287 >> 31)); 
} 


if (1048575 >> 31 !== 0) { 
  throw new Test262Error('#1013: 1048575 >> 31 === 0. Actual: ' + (1048575 >> 31)); 
} 


if (2097151 >> 31 !== 0) { 
  throw new Test262Error('#1014: 2097151 >> 31 === 0. Actual: ' + (2097151 >> 31)); 
} 


if (4194303 >> 31 !== 0) { 
  throw new Test262Error('#1015: 4194303 >> 31 === 0. Actual: ' + (4194303 >> 31)); 
} 


if (8388607 >> 31 !== 0) { 
  throw new Test262Error('#1016: 8388607 >> 31 === 0. Actual: ' + (8388607 >> 31)); 
} 


if (16777215 >> 31 !== 0) { 
  throw new Test262Error('#1017: 16777215 >> 31 === 0. Actual: ' + (16777215 >> 31)); 
} 


if (33554431 >> 31 !== 0) { 
  throw new Test262Error('#1018: 33554431 >> 31 === 0. Actual: ' + (33554431 >> 31)); 
} 


if (67108863 >> 31 !== 0) { 
  throw new Test262Error('#1019: 67108863 >> 31 === 0. Actual: ' + (67108863 >> 31)); 
} 


if (134217727 >> 31 !== 0) { 
  throw new Test262Error('#1020: 134217727 >> 31 === 0. Actual: ' + (134217727 >> 31)); 
} 


if (268435455 >> 31 !== 0) { 
  throw new Test262Error('#1021: 268435455 >> 31 === 0. Actual: ' + (268435455 >> 31)); 
} 


if (536870911 >> 31 !== 0) { 
  throw new Test262Error('#1022: 536870911 >> 31 === 0. Actual: ' + (536870911 >> 31)); 
} 


if (1073741823 >> 31 !== 0) { 
  throw new Test262Error('#1023: 1073741823 >> 31 === 0. Actual: ' + (1073741823 >> 31)); 
}
