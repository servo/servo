/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert } from '../util/util.js'; /** Metadata about tests (that can't be derived at runtime). */

export function loadMetadataForSuite(suiteDir) {
  assert(typeof require !== 'undefined', 'loadMetadataForSuite is only implemented on Node');
  const fs = require('fs');

  const metadataFile = `${suiteDir}/listing_meta.json`;
  if (!fs.existsSync(metadataFile)) {
    return null;
  }

  const metadata = JSON.parse(fs.readFileSync(metadataFile, 'utf8'));
  return metadata;
}
