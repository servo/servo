(module
 (type $0 (func (param externref i32)))
 (import "../clearCalls.js" "clearThruJS" (func $assembly/index/clearThruJS (param externref i32)))
 (import "../clearCalls.js" "clearNoJS" (func $assembly/index/clearNoJS (param externref i32)))
 (memory $0 0)
 (export "clearManyTimesThruJS" (func $assembly/index/clearManyTimesThruJS))
 (export "clearManyTimesNoJS" (func $assembly/index/clearManyTimesNoJS))
 (export "memory" (memory $0))
 (func $assembly/index/clearManyTimesThruJS (param $0 externref) (param $1 i32)
  (local $2 i32)
  loop $for-loop|0
   local.get $1
   local.get $2
   i32.gt_s
   if
    local.get $0
    i32.const 16384
    call $assembly/index/clearThruJS
    local.get $2
    i32.const 1
    i32.add
    local.set $2
    br $for-loop|0
   end
  end
 )
 (func $assembly/index/clearManyTimesNoJS (param $0 externref) (param $1 i32)
  (local $2 i32)
  loop $for-loop|0
   local.get $1
   local.get $2
   i32.gt_s
   if
    local.get $0
    i32.const 16384
    call $assembly/index/clearNoJS
    local.get $2
    i32.const 1
    i32.add
    local.set $2
    br $for-loop|0
   end
  end
 )
)
