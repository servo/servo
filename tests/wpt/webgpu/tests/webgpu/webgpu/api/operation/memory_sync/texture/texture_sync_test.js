/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { GPUConst } from '../../../../constants.js';
export const kAllWriteOps = [
  'write-texture',
  'b2t-copy',
  't2t-copy',
  'storage',
  'attachment-store',
  'attachment-resolve',
];

export const kAllReadOps = ['t2b-copy', 't2t-copy', 'sample'];

/**
 * Mapping of Op to the OperationContext(s) it is valid in
 */
export const kOpInfo = {
  'write-texture': {
    contexts: ['queue'],
    readUsage: 0,
    writeUsage: GPUConst.TextureUsage.COPY_DST,
  },
  'b2t-copy': {
    contexts: ['command-encoder'],
    readUsage: 0,
    writeUsage: GPUConst.TextureUsage.COPY_DST,
  },
  't2t-copy': {
    contexts: ['command-encoder'],
    readUsage: GPUConst.TextureUsage.COPY_SRC,
    writeUsage: GPUConst.TextureUsage.COPY_DST,
  },
  't2b-copy': {
    contexts: ['command-encoder'],
    readUsage: GPUConst.TextureUsage.COPY_SRC,
    writeUsage: 0,
  },
  storage: {
    contexts: ['compute-pass-encoder', 'render-pass-encoder', 'render-bundle-encoder'],
    readUsage: 0,
    writeUsage: GPUConst.TextureUsage.STORAGE,
  },
  sample: {
    contexts: ['compute-pass-encoder', 'render-pass-encoder', 'render-bundle-encoder'],
    readUsage: GPUConst.TextureUsage.SAMPLED,
    writeUsage: 0,
  },
  'attachment-store': {
    contexts: ['command-encoder'],
    readUsage: 0,
    writeUsage: GPUConst.TextureUsage.RENDER_ATTACHMENT,
  },
  'attachment-resolve': {
    contexts: ['command-encoder'],
    readUsage: 0,
    writeUsage: GPUConst.TextureUsage.RENDER_ATTACHMENT,
  },
};

export function checkOpsValidForContext(ops, context) {
  const valid =
    kOpInfo[ops[0]].contexts.includes(context[0]) && kOpInfo[ops[1]].contexts.includes(context[1]);
  if (!valid) return false;

  if (
    context[0] === 'render-bundle-encoder' ||
    context[0] === 'render-pass-encoder' ||
    context[1] === 'render-bundle-encoder' ||
    context[1] === 'render-pass-encoder'
  ) {
    // In a render pass, it is invalid to use a resource as both writable and another usage.
    // Also, for storage+storage usage, the application is opting into racy behavior.
    // The storage+storage case is also skipped as the results cannot be reliably tested.
    const checkImpl = (op1, op2) => {
      switch (op1) {
        case 'attachment-resolve':
        case 'attachment-store':
        case 'storage':
          switch (op2) {
            case 'attachment-resolve':
            case 'attachment-store':
            case 'storage':
            case 'sample':
              // Write+other, or racy.
              return false;
            case 'b2t-copy':
            case 't2b-copy':
            case 't2t-copy':
            case 'write-texture':
              // These don't occur in a render pass.
              return true;
          }

          break;
        case 'b2t-copy':
        case 'sample':
        case 't2b-copy':
        case 't2t-copy':
        case 'write-texture':
          // These are not write usages, or don't occur in a render pass.
          break;
      }

      return true;
    };
    return checkImpl(ops[0], ops[1]) && checkImpl(ops[1], ops[0]);
  }
  return true;
}
