// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// When imported, this file will ensure that a linking error happens by
// importing a non-existent binding.
// It can be used to assert that there is a linking error, which means
// that there are no parsing errors.

import { nonExistent } from "./ensure-linking-error_FIXTURE.js";
