// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escapes the initial character if it is a decimal digit or an ASCII letter
info: |
  RegExp.escape ( string )

  1. If S is not a String, throw a TypeError exception.
  2. Let escaped be the empty String.
  3. Let cpList be StringToCodePoints(S).
  4. For each code point c in cpList, do
    a. If escaped is the empty String, and c is matched by DecimalDigit or AsciiLetter, then
      i. NOTE: Escaping a leading digit ensures that output corresponds with pattern text which may be used after a \0 character escape or a DecimalEscape such as \1 and still match S rather than be interpreted as an extension of the preceding escape sequence. Escaping a leading ASCII letter does the same for the context after \c.
      ii. Let numericValue be the numeric value of c.
      iii. Let hex be Number::toString(𝔽(numericValue), 16).
      iv. Assert: The length of hex is 2.
      v. Set escaped to the string-concatenation of the code unit 0x005C (REVERSE SOLIDUS), "x", and hex.
    b. Else,
      i. Set escaped to the string-concatenation of escaped and EncodeForRegExpEscape(c).
  5. Return escaped.

features: [RegExp.escape]
---*/

// Escaping initial digits
assert.sameValue(RegExp.escape('1111'), '\\x31111', 'Initial decimal digit 1 is escaped');
assert.sameValue(RegExp.escape('2222'), '\\x32222', 'Initial decimal digit 2 is escaped');
assert.sameValue(RegExp.escape('3333'), '\\x33333', 'Initial decimal digit 3 is escaped');
assert.sameValue(RegExp.escape('4444'), '\\x34444', 'Initial decimal digit 4 is escaped');
assert.sameValue(RegExp.escape('5555'), '\\x35555', 'Initial decimal digit 5 is escaped');
assert.sameValue(RegExp.escape('6666'), '\\x36666', 'Initial decimal digit 6 is escaped');
assert.sameValue(RegExp.escape('7777'), '\\x37777', 'Initial decimal digit 7 is escaped');
assert.sameValue(RegExp.escape('8888'), '\\x38888', 'Initial decimal digit 8 is escaped');
assert.sameValue(RegExp.escape('9999'), '\\x39999', 'Initial decimal digit 9 is escaped');
assert.sameValue(RegExp.escape('0000'), '\\x30000', 'Initial decimal digit 0 is escaped');

// Escaping initial ASCII letters
assert.sameValue(RegExp.escape('aaa'), '\\x61aa', 'Initial ASCII letter a is escaped');
assert.sameValue(RegExp.escape('bbb'), '\\x62bb', 'Initial ASCII letter b is escaped');
assert.sameValue(RegExp.escape('ccc'), '\\x63cc', 'Initial ASCII letter c is escaped');
assert.sameValue(RegExp.escape('ddd'), '\\x64dd', 'Initial ASCII letter d is escaped');
assert.sameValue(RegExp.escape('eee'), '\\x65ee', 'Initial ASCII letter e is escaped');
assert.sameValue(RegExp.escape('fff'), '\\x66ff', 'Initial ASCII letter f is escaped');
assert.sameValue(RegExp.escape('ggg'), '\\x67gg', 'Initial ASCII letter g is escaped');
assert.sameValue(RegExp.escape('hhh'), '\\x68hh', 'Initial ASCII letter h is escaped');
assert.sameValue(RegExp.escape('iii'), '\\x69ii', 'Initial ASCII letter i is escaped');
assert.sameValue(RegExp.escape('jjj'), '\\x6ajj', 'Initial ASCII letter j is escaped');
assert.sameValue(RegExp.escape('kkk'), '\\x6bkk', 'Initial ASCII letter k is escaped');
assert.sameValue(RegExp.escape('lll'), '\\x6cll', 'Initial ASCII letter l is escaped');
assert.sameValue(RegExp.escape('mmm'), '\\x6dmm', 'Initial ASCII letter m is escaped');
assert.sameValue(RegExp.escape('nnn'), '\\x6enn', 'Initial ASCII letter n is escaped');
assert.sameValue(RegExp.escape('ooo'), '\\x6foo', 'Initial ASCII letter o is escaped');
assert.sameValue(RegExp.escape('ppp'), '\\x70pp', 'Initial ASCII letter p is escaped');
assert.sameValue(RegExp.escape('qqq'), '\\x71qq', 'Initial ASCII letter q is escaped');
assert.sameValue(RegExp.escape('rrr'), '\\x72rr', 'Initial ASCII letter r is escaped');
assert.sameValue(RegExp.escape('sss'), '\\x73ss', 'Initial ASCII letter s is escaped');
assert.sameValue(RegExp.escape('ttt'), '\\x74tt', 'Initial ASCII letter t is escaped');
assert.sameValue(RegExp.escape('uuu'), '\\x75uu', 'Initial ASCII letter u is escaped');
assert.sameValue(RegExp.escape('vvv'), '\\x76vv', 'Initial ASCII letter v is escaped');
assert.sameValue(RegExp.escape('www'), '\\x77ww', 'Initial ASCII letter w is escaped');
assert.sameValue(RegExp.escape('xxx'), '\\x78xx', 'Initial ASCII letter x is escaped');
assert.sameValue(RegExp.escape('yyy'), '\\x79yy', 'Initial ASCII letter y is escaped');
assert.sameValue(RegExp.escape('zzz'), '\\x7azz', 'Initial ASCII letter z is escaped');
assert.sameValue(RegExp.escape('AAA'), '\\x41AA', 'Initial ASCII letter A is escaped');
assert.sameValue(RegExp.escape('BBB'), '\\x42BB', 'Initial ASCII letter B is escaped');
assert.sameValue(RegExp.escape('CCC'), '\\x43CC', 'Initial ASCII letter C is escaped');
assert.sameValue(RegExp.escape('DDD'), '\\x44DD', 'Initial ASCII letter D is escaped');
assert.sameValue(RegExp.escape('EEE'), '\\x45EE', 'Initial ASCII letter E is escaped');
assert.sameValue(RegExp.escape('FFF'), '\\x46FF', 'Initial ASCII letter F is escaped');
assert.sameValue(RegExp.escape('GGG'), '\\x47GG', 'Initial ASCII letter G is escaped');
assert.sameValue(RegExp.escape('HHH'), '\\x48HH', 'Initial ASCII letter H is escaped');
assert.sameValue(RegExp.escape('III'), '\\x49II', 'Initial ASCII letter I is escaped');
assert.sameValue(RegExp.escape('JJJ'), '\\x4aJJ', 'Initial ASCII letter J is escaped');
assert.sameValue(RegExp.escape('KKK'), '\\x4bKK', 'Initial ASCII letter K is escaped');
assert.sameValue(RegExp.escape('LLL'), '\\x4cLL', 'Initial ASCII letter L is escaped');
assert.sameValue(RegExp.escape('MMM'), '\\x4dMM', 'Initial ASCII letter M is escaped');
assert.sameValue(RegExp.escape('NNN'), '\\x4eNN', 'Initial ASCII letter N is escaped');
assert.sameValue(RegExp.escape('OOO'), '\\x4fOO', 'Initial ASCII letter O is escaped');
assert.sameValue(RegExp.escape('PPP'), '\\x50PP', 'Initial ASCII letter P is escaped');
assert.sameValue(RegExp.escape('QQQ'), '\\x51QQ', 'Initial ASCII letter Q is escaped');
assert.sameValue(RegExp.escape('RRR'), '\\x52RR', 'Initial ASCII letter R is escaped');
assert.sameValue(RegExp.escape('SSS'), '\\x53SS', 'Initial ASCII letter S is escaped');
assert.sameValue(RegExp.escape('TTT'), '\\x54TT', 'Initial ASCII letter T is escaped');
assert.sameValue(RegExp.escape('UUU'), '\\x55UU', 'Initial ASCII letter U is escaped');
assert.sameValue(RegExp.escape('VVV'), '\\x56VV', 'Initial ASCII letter V is escaped');
assert.sameValue(RegExp.escape('WWW'), '\\x57WW', 'Initial ASCII letter W is escaped');
assert.sameValue(RegExp.escape('XXX'), '\\x58XX', 'Initial ASCII letter X is escaped');
assert.sameValue(RegExp.escape('YYY'), '\\x59YY', 'Initial ASCII letter Y is escaped');
assert.sameValue(RegExp.escape('ZZZ'), '\\x5aZZ', 'Initial ASCII letter Z is escaped');

// Mixed case with special characters
assert.sameValue(RegExp.escape('1+1'), '\\x31\\+1', 'Initial decimal digit 1 with special character is escaped');
assert.sameValue(RegExp.escape('2+2'), '\\x32\\+2', 'Initial decimal digit 2 with special character is escaped');
assert.sameValue(RegExp.escape('3+3'), '\\x33\\+3', 'Initial decimal digit 3 with special character is escaped');
assert.sameValue(RegExp.escape('4+4'), '\\x34\\+4', 'Initial decimal digit 4 with special character is escaped');
assert.sameValue(RegExp.escape('5+5'), '\\x35\\+5', 'Initial decimal digit 5 with special character is escaped');
assert.sameValue(RegExp.escape('6+6'), '\\x36\\+6', 'Initial decimal digit 6 with special character is escaped');
assert.sameValue(RegExp.escape('7+7'), '\\x37\\+7', 'Initial decimal digit 7 with special character is escaped');
assert.sameValue(RegExp.escape('8+8'), '\\x38\\+8', 'Initial decimal digit 8 with special character is escaped');
assert.sameValue(RegExp.escape('9+9'), '\\x39\\+9', 'Initial decimal digit 9 with special character is escaped');
assert.sameValue(RegExp.escape('0+0'), '\\x30\\+0', 'Initial decimal digit 0 with special character is escaped');

assert.sameValue(RegExp.escape('a*a'), '\\x61\\*a', 'Initial ASCII letter a with special character is escaped');
assert.sameValue(RegExp.escape('b*b'), '\\x62\\*b', 'Initial ASCII letter b with special character is escaped');
assert.sameValue(RegExp.escape('c*c'), '\\x63\\*c', 'Initial ASCII letter c with special character is escaped');
assert.sameValue(RegExp.escape('d*d'), '\\x64\\*d', 'Initial ASCII letter d with special character is escaped');
assert.sameValue(RegExp.escape('e*e'), '\\x65\\*e', 'Initial ASCII letter e with special character is escaped');
assert.sameValue(RegExp.escape('f*f'), '\\x66\\*f', 'Initial ASCII letter f with special character is escaped');
assert.sameValue(RegExp.escape('g*g'), '\\x67\\*g', 'Initial ASCII letter g with special character is escaped');
assert.sameValue(RegExp.escape('h*h'), '\\x68\\*h', 'Initial ASCII letter h with special character is escaped');
assert.sameValue(RegExp.escape('i*i'), '\\x69\\*i', 'Initial ASCII letter i with special character is escaped');
assert.sameValue(RegExp.escape('j*j'), '\\x6a\\*j', 'Initial ASCII letter j with special character is escaped');
assert.sameValue(RegExp.escape('k*k'), '\\x6b\\*k', 'Initial ASCII letter k with special character is escaped');
assert.sameValue(RegExp.escape('l*l'), '\\x6c\\*l', 'Initial ASCII letter l with special character is escaped');
assert.sameValue(RegExp.escape('m*m'), '\\x6d\\*m', 'Initial ASCII letter m with special character is escaped');
assert.sameValue(RegExp.escape('n*n'), '\\x6e\\*n', 'Initial ASCII letter n with special character is escaped');
assert.sameValue(RegExp.escape('o*o'), '\\x6f\\*o', 'Initial ASCII letter o with special character is escaped');
assert.sameValue(RegExp.escape('p*p'), '\\x70\\*p', 'Initial ASCII letter p with special character is escaped');
assert.sameValue(RegExp.escape('q*q'), '\\x71\\*q', 'Initial ASCII letter q with special character is escaped');
assert.sameValue(RegExp.escape('r*r'), '\\x72\\*r', 'Initial ASCII letter r with special character is escaped');
assert.sameValue(RegExp.escape('s*s'), '\\x73\\*s', 'Initial ASCII letter s with special character is escaped');
assert.sameValue(RegExp.escape('t*t'), '\\x74\\*t', 'Initial ASCII letter t with special character is escaped');
assert.sameValue(RegExp.escape('u*u'), '\\x75\\*u', 'Initial ASCII letter u with special character is escaped');
assert.sameValue(RegExp.escape('v*v'), '\\x76\\*v', 'Initial ASCII letter v with special character is escaped');
assert.sameValue(RegExp.escape('w*w'), '\\x77\\*w', 'Initial ASCII letter w with special character is escaped');
assert.sameValue(RegExp.escape('x*x'), '\\x78\\*x', 'Initial ASCII letter x with special character is escaped');
assert.sameValue(RegExp.escape('y*y'), '\\x79\\*y', 'Initial ASCII letter y with special character is escaped');
assert.sameValue(RegExp.escape('z*z'), '\\x7a\\*z', 'Initial ASCII letter z with special character is escaped');
assert.sameValue(RegExp.escape('A*A'), '\\x41\\*A', 'Initial ASCII letter A with special character is escaped');
assert.sameValue(RegExp.escape('B*B'), '\\x42\\*B', 'Initial ASCII letter B with special character is escaped');
assert.sameValue(RegExp.escape('C*C'), '\\x43\\*C', 'Initial ASCII letter C with special character is escaped');
assert.sameValue(RegExp.escape('D*D'), '\\x44\\*D', 'Initial ASCII letter D with special character is escaped');
assert.sameValue(RegExp.escape('E*E'), '\\x45\\*E', 'Initial ASCII letter E with special character is escaped');
assert.sameValue(RegExp.escape('F*F'), '\\x46\\*F', 'Initial ASCII letter F with special character is escaped');
assert.sameValue(RegExp.escape('G*G'), '\\x47\\*G', 'Initial ASCII letter G with special character is escaped');
assert.sameValue(RegExp.escape('H*H'), '\\x48\\*H', 'Initial ASCII letter H with special character is escaped');
assert.sameValue(RegExp.escape('I*I'), '\\x49\\*I', 'Initial ASCII letter I with special character is escaped');
assert.sameValue(RegExp.escape('J*J'), '\\x4a\\*J', 'Initial ASCII letter J with special character is escaped');
assert.sameValue(RegExp.escape('K*K'), '\\x4b\\*K', 'Initial ASCII letter K with special character is escaped');
assert.sameValue(RegExp.escape('L*L'), '\\x4c\\*L', 'Initial ASCII letter L with special character is escaped');
assert.sameValue(RegExp.escape('M*M'), '\\x4d\\*M', 'Initial ASCII letter M with special character is escaped');
assert.sameValue(RegExp.escape('N*N'), '\\x4e\\*N', 'Initial ASCII letter N with special character is escaped');
assert.sameValue(RegExp.escape('O*O'), '\\x4f\\*O', 'Initial ASCII letter O with special character is escaped');
assert.sameValue(RegExp.escape('P*P'), '\\x50\\*P', 'Initial ASCII letter P with special character is escaped');
assert.sameValue(RegExp.escape('Q*Q'), '\\x51\\*Q', 'Initial ASCII letter Q with special character is escaped');
assert.sameValue(RegExp.escape('R*R'), '\\x52\\*R', 'Initial ASCII letter R with special character is escaped');
assert.sameValue(RegExp.escape('S*S'), '\\x53\\*S', 'Initial ASCII letter S with special character is escaped');
assert.sameValue(RegExp.escape('T*T'), '\\x54\\*T', 'Initial ASCII letter T with special character is escaped');
assert.sameValue(RegExp.escape('U*U'), '\\x55\\*U', 'Initial ASCII letter U with special character is escaped');
assert.sameValue(RegExp.escape('V*V'), '\\x56\\*V', 'Initial ASCII letter V with special character is escaped');
assert.sameValue(RegExp.escape('W*W'), '\\x57\\*W', 'Initial ASCII letter W with special character is escaped');
assert.sameValue(RegExp.escape('X*X'), '\\x58\\*X', 'Initial ASCII letter X with special character is escaped');
assert.sameValue(RegExp.escape('Y*Y'), '\\x59\\*Y', 'Initial ASCII letter Y with special character is escaped');
assert.sameValue(RegExp.escape('Z*Z'), '\\x5a\\*Z', 'Initial ASCII letter Z with special character is escaped');
