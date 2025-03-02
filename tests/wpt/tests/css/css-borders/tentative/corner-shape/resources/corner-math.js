// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
/**
 * @param {number} s
 * @param {number} t
 * @returns {x: number, y: number}
 */
function se(s, t = 0.5) {
  const curvature = Math.pow(2, s);
  const x = superellipse_at(curvature);
  const y = superellipse_at(curvature, 1 - t);
  return {x, y};
}

/**
 *
 * @param {number} k
 * @returns Array<[number, number]>
 */
function control_points_for_superellipse(k) {
  const p = [
    1.2430920942724248, 2.010479023614843, 0.32922901179443753,
    0.2823023142212073, 1.3473704261055421, 2.9149468637949814,
    0.9106507102917086
  ];

  const s = Math.log2(k);
  const absS = Math.abs(s);
  const slope =
      p[0] + (p[6] - p[0]) * 0.5 * (1 + Math.tanh(p[5] * (absS - p[1])));
  const base = 1 / (1 + Math.exp(-slope * (0 - p[1])));
  const logistic = 1 / (1 + Math.exp(-slope * (absS - p[1])));

  const a = (logistic - base) / (1 - base);
  const b = p[2] * Math.exp(-p[3] * absS ** p[4]);

  const P3 = se(absS, 0.5);
  const P1 = {x: a, y: 1};
  const P5 = {x: 1, y: a};

  if (s < 0) {
    [P1.x, P1.y] = [1 - P1.y, 1 - P1.x];
    [P3.x, P3.y] = [1 - P3.y, 1 - P3.x];
    [P5.x, P5.y] = [1 - P5.y, 1 - P5.x];
  }

  const P2 = {x: P3.x - b, y: P3.y + b};
  const P4 = {x: P3.x + b, y: P3.y - b};
  return [P1, P2, P3, P4, P5].map(({x, y}) => [x, 1 - y]);
}
