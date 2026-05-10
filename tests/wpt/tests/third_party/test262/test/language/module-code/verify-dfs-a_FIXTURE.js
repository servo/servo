// Copyright (C) 2020 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import { evaluated, check } from './verify-dfs.js';

check(import('./verify-dfs-b_FIXTURE.js'));

evaluated('A');
