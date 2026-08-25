/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://streams.spec.whatwg.org/#transformer-api
 */

[GenerateInit]
dictionary Transformer {
  TransformerStartCallback start;
  TransformerTransformCallback transform;
  TransformerFlushCallback flush;
  TransformerCancelCallback cancel;
  any readableType;
  any writableType;
};

callback TransformerStartCallback = any (TransformStreamDefaultController controller);
callback TransformerFlushCallback = Promise<undefined> (TransformStreamDefaultController controller);
callback TransformerTransformCallback = Promise<undefined> (any chunk, TransformStreamDefaultController controller);
callback TransformerCancelCallback = Promise<undefined> (any reason);
