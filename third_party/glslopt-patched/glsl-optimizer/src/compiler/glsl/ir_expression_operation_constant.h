   switch (this->operation) {
   case ir_unop_bit_not:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = ~ op[0]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = ~ op[0]->value.i[c];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = ~ op[0]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = ~ op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_logic_not:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.b[c] = !op[0]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_neg:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = -((int) op[0]->value.u[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = -op[0]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = -op[0]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = -op[0]->value.d[c];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = -((int64_t) op[0]->value.u64[c]);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = -op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_abs:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c] < 0 ? -op[0]->value.i[c] : op[0]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = fabsf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = fabs(op[0]->value.d[c]);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c] < 0 ? -op[0]->value.i64[c] : op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_sign:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.i[c] = (op[0]->value.i[c] > 0) - (op[0]->value.i[c] < 0);
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = float((op[0]->value.f[c] > 0.0F) - (op[0]->value.f[c] < 0.0F));
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = double((op[0]->value.d[c] > 0.0) - (op[0]->value.d[c] < 0.0));
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = (op[0]->value.i64[c] > 0) - (op[0]->value.i64[c] < 0);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_rcp:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 1.0F / op[0]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = 1.0 / op[0]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_rsq:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 1.0F / sqrtf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = 1.0 / sqrt(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_sqrt:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = sqrtf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = sqrt(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_exp:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = expf(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_log:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = logf(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_exp2:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = exp2f(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_log2:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = log2f(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.i[c] = (int) op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2u:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.u[c] = (unsigned) op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i2f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.f[c] = (float) op[0]->value.i[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2b:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.b[c] = op[0]->value.f[c] != 0.0F ? true : false;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_b2f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.f[c] = op[0]->value.b[c] ? 1.0F : 0.0F;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_b2f16:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.f[c] = op[0]->value.b[c] ? 1.0F : 0.0F;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i2b:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.b[c] = op[0]->value.u[c] ? true : false;
            break;
         case GLSL_TYPE_INT:
            data.b[c] = op[0]->value.i[c] ? true : false;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_b2i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.i[c] = op[0]->value.b[c] ? 1 : 0;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u2f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.f[c] = (float) op[0]->value.u[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i2u:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.u[c] = op[0]->value.i[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u2i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.i[c] = op[0]->value.u[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_d2f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.f[c] = op[0]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.d[c] = op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2f16:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2fmp:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f162f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_d2i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.i[c] = op[0]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i2d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.d[c] = op[0]->value.i[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_d2u:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.u[c] = op[0]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u2d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.d[c] = op[0]->value.u[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_d2b:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.b[c] = op[0]->value.d[c] != 0.0;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f162b:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.b[c] = op[0]->value.f[c] != 0.0;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_i2f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.f[c] = bitcast_u2f(op[0]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_f2i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.i[c] = bitcast_f2u(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_u2f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.f[c] = bitcast_u2f(op[0]->value.u[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_f2u:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.u[c] = bitcast_f2u(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_u642d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT64:
            data.d[c] = bitcast_u642d(op[0]->value.u64[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_i642d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.d[c] = bitcast_i642d(op[0]->value.i64[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_d2u64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.u64[c] = bitcast_d2u64(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bitcast_d2i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.i64[c] = bitcast_d2i64(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i642i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.i[c] = op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u642i:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT64:
            data.i[c] = op[0]->value.u64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i642u:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.u[c] = op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u642u:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT64:
            data.u[c] = op[0]->value.u64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i642b:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.b[c] = op[0]->value.i64[c] != 0;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i642f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.f[c] = op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u642f:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT64:
            data.f[c] = op[0]->value.u64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i642d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.d[c] = op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u642d:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT64:
            data.d[c] = op[0]->value.u64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i2i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.i64[c] = op[0]->value.i[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u2i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.i64[c] = op[0]->value.u[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_b2i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.i64[c] = op[0]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.i64[c] = op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_d2i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.i64[c] = op[0]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i2u64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT:
            data.u64[c] = op[0]->value.i[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u2u64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u64[c] = op[0]->value.u[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_f2u64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.u64[c] = op[0]->value.f[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_d2u64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_DOUBLE:
            data.u64[c] = op[0]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_u642i64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT64:
            data.i64[c] = op[0]->value.u64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_i642u64:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_INT64:
            data.u64[c] = op[0]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_trunc:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = truncf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = trunc(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_ceil:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = ceilf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = ceil(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_floor:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = floorf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = floor(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_fract:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c] - floorf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c] - floor(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_round_even:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = _mesa_roundevenf(op[0]->value.f[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = _mesa_roundeven(op[0]->value.d[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_sin:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = sinf(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_cos:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = cosf(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_atan:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = atan(op[0]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_dFdx:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 0.0f;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_dFdx_coarse:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 0.0f;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_dFdx_fine:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 0.0f;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_dFdy:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 0.0f;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_dFdy_coarse:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 0.0f;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_dFdy_fine:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = 0.0f;
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_pack_snorm_2x16:
      switch (op[0]->type->base_type) {
      case GLSL_TYPE_FLOAT:
         data.u[0] = pack_2x16(pack_snorm_1x16, op[0]->value.f[0], op[0]->value.f[1]);
         break;
      default:
         unreachable("invalid type");
      }
      break;

   case ir_unop_pack_snorm_4x8:
      switch (op[0]->type->base_type) {
      case GLSL_TYPE_FLOAT:
         data.u[0] = pack_4x8(pack_snorm_1x8, op[0]->value.f[0], op[0]->value.f[1], op[0]->value.f[2], op[0]->value.f[3]);
         break;
      default:
         unreachable("invalid type");
      }
      break;

   case ir_unop_pack_unorm_2x16:
      switch (op[0]->type->base_type) {
      case GLSL_TYPE_FLOAT:
         data.u[0] = pack_2x16(pack_unorm_1x16, op[0]->value.f[0], op[0]->value.f[1]);
         break;
      default:
         unreachable("invalid type");
      }
      break;

   case ir_unop_pack_unorm_4x8:
      switch (op[0]->type->base_type) {
      case GLSL_TYPE_FLOAT:
         data.u[0] = pack_4x8(pack_unorm_1x8, op[0]->value.f[0], op[0]->value.f[1], op[0]->value.f[2], op[0]->value.f[3]);
         break;
      default:
         unreachable("invalid type");
      }
      break;

   case ir_unop_pack_half_2x16:
      switch (op[0]->type->base_type) {
      case GLSL_TYPE_FLOAT:
         data.u[0] = pack_2x16(pack_half_1x16, op[0]->value.f[0], op[0]->value.f[1]);
         break;
      default:
         unreachable("invalid type");
      }
      break;

   case ir_unop_unpack_snorm_2x16:
      unpack_2x16(unpack_snorm_1x16, op[0]->value.u[0], &data.f[0], &data.f[1]);
      break;

   case ir_unop_unpack_snorm_4x8:
      unpack_4x8(unpack_snorm_1x8, op[0]->value.u[0], &data.f[0], &data.f[1], &data.f[2], &data.f[3]);
      break;

   case ir_unop_unpack_unorm_2x16:
      unpack_2x16(unpack_unorm_1x16, op[0]->value.u[0], &data.f[0], &data.f[1]);
      break;

   case ir_unop_unpack_unorm_4x8:
      unpack_4x8(unpack_unorm_1x8, op[0]->value.u[0], &data.f[0], &data.f[1], &data.f[2], &data.f[3]);
      break;

   case ir_unop_unpack_half_2x16:
      unpack_2x16(unpack_half_1x16, op[0]->value.u[0], &data.f[0], &data.f[1]);
      break;

   case ir_unop_bitfield_reverse:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = bitfield_reverse(op[0]->value.u[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = bitfield_reverse(op[0]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_bit_count:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.i[c] = util_bitcount(op[0]->value.u[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = util_bitcount(op[0]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_find_msb:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.i[c] = find_msb_uint(op[0]->value.u[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = find_msb_int(op[0]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_find_lsb:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.i[c] = find_msb_uint(op[0]->value.u[c] & -op[0]->value.u[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = find_msb_uint(op[0]->value.i[c] & -op[0]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_clz:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = (unsigned)(31 - find_msb_uint(op[0]->value.u[c]));
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_saturate:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = CLAMP(op[0]->value.f[c], 0.0f, 1.0f);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_unop_pack_double_2x32:
      data.u64[0] = pack_2x32(op[0]->value.u[0], op[0]->value.u[1]);
      break;

   case ir_unop_unpack_double_2x32:
      unpack_2x32(op[0]->value.u64[0], &data.u[0], &data.u[1]);
      break;

   case ir_unop_pack_sampler_2x32:
      data.u64[0] = pack_2x32(op[0]->value.u[0], op[0]->value.u[1]);
      break;

   case ir_unop_pack_image_2x32:
      data.u64[0] = pack_2x32(op[0]->value.u[0], op[0]->value.u[1]);
      break;

   case ir_unop_unpack_sampler_2x32:
      unpack_2x32(op[0]->value.u64[0], &data.u[0], &data.u[1]);
      break;

   case ir_unop_unpack_image_2x32:
      unpack_2x32(op[0]->value.u64[0], &data.u[0], &data.u[1]);
      break;

   case ir_unop_pack_int_2x32:
      data.u64[0] = pack_2x32(op[0]->value.u[0], op[0]->value.u[1]);
      break;

   case ir_unop_pack_uint_2x32:
      data.u64[0] = pack_2x32(op[0]->value.u[0], op[0]->value.u[1]);
      break;

   case ir_unop_unpack_int_2x32:
      unpack_2x32(op[0]->value.u64[0], &data.u[0], &data.u[1]);
      break;

   case ir_unop_unpack_uint_2x32:
      unpack_2x32(op[0]->value.u64[0], &data.u[0], &data.u[1]);
      break;

   case ir_binop_add:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] + op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] + op[1]->value.i[c1];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c0] + op[1]->value.f[c1];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c0] + op[1]->value.d[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] + op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] + op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_sub:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] - op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] - op[1]->value.i[c1];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c0] - op[1]->value.f[c1];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c0] - op[1]->value.d[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] - op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] - op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_add_sat:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = (op[0]->value.u[c] + op[1]->value.u[c]) < op[0]->value.u[c] ? UINT32_MAX : (op[0]->value.u[c] + op[1]->value.u[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = iadd_saturate(op[0]->value.i[c], op[1]->value.i[c]);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = (op[0]->value.u64[c] + op[1]->value.u64[c]) < op[0]->value.u64[c] ? UINT64_MAX : (op[0]->value.u64[c] + op[1]->value.u64[c]);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = iadd64_saturate(op[0]->value.i64[c], op[1]->value.i64[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_sub_sat:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = (op[1]->value.u[c] > op[0]->value.u[c]) ? 0 : op[0]->value.u[c] - op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = isub_saturate(op[0]->value.i[c], op[1]->value.i[c]);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = (op[1]->value.u64[c] > op[0]->value.u64[c]) ? 0 : op[0]->value.u64[c] - op[1]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = isub64_saturate(op[0]->value.i64[c], op[1]->value.i64[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_abs_sub:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = (op[1]->value.u[c] > op[0]->value.u[c]) ? op[1]->value.u[c] - op[0]->value.u[c] : op[0]->value.u[c] - op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = (op[1]->value.i[c] > op[0]->value.i[c]) ? (unsigned)op[1]->value.i[c] - (unsigned)op[0]->value.i[c] : (unsigned)op[0]->value.i[c] - (unsigned)op[1]->value.i[c];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = (op[1]->value.u64[c] > op[0]->value.u64[c]) ? op[1]->value.u64[c] - op[0]->value.u64[c] : op[0]->value.u64[c] - op[1]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = (op[1]->value.i64[c] > op[0]->value.i64[c]) ? (uint64_t)op[1]->value.i64[c] - (uint64_t)op[0]->value.i64[c] : (uint64_t)op[0]->value.i64[c] - (uint64_t)op[1]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_avg:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = (op[0]->value.u[c] >> 1) + (op[1]->value.u[c] >> 1) + ((op[0]->value.u[c] & op[1]->value.u[c]) & 1);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = (op[0]->value.i[c] >> 1) + (op[1]->value.i[c] >> 1) + ((op[0]->value.i[c] & op[1]->value.i[c]) & 1);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = (op[0]->value.u64[c] >> 1) + (op[1]->value.u64[c] >> 1) + ((op[0]->value.u64[c] & op[1]->value.u64[c]) & 1);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = (op[0]->value.i64[c] >> 1) + (op[1]->value.i64[c] >> 1) + ((op[0]->value.i64[c] & op[1]->value.i64[c]) & 1);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_avg_round:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = (op[0]->value.u[c] >> 1) + (op[1]->value.u[c] >> 1) + ((op[0]->value.u[c] | op[1]->value.u[c]) & 1);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = (op[0]->value.i[c] >> 1) + (op[1]->value.i[c] >> 1) + ((op[0]->value.i[c] | op[1]->value.i[c]) & 1);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = (op[0]->value.u64[c] >> 1) + (op[1]->value.u64[c] >> 1) + ((op[0]->value.u64[c] | op[1]->value.u64[c]) & 1);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = (op[0]->value.i64[c] >> 1) + (op[1]->value.i64[c] >> 1) + ((op[0]->value.i64[c] | op[1]->value.i64[c]) & 1);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_mul:
      /* Check for equal types, or unequal types involving scalars */
      if ((op[0]->type == op[1]->type && !op[0]->type->is_matrix())
          || op0_scalar || op1_scalar) {
         for (unsigned c = 0, c0 = 0, c1 = 0;
              c < components;
              c0 += c0_inc, c1 += c1_inc, c++) {

            switch (op[0]->type->base_type) {
            case GLSL_TYPE_UINT:
               data.u[c] = op[0]->value.u[c0] * op[1]->value.u[c1];
               break;
            case GLSL_TYPE_INT:
               data.i[c] = op[0]->value.i[c0] * op[1]->value.i[c1];
               break;
            case GLSL_TYPE_FLOAT:
               data.f[c] = op[0]->value.f[c0] * op[1]->value.f[c1];
               break;
            case GLSL_TYPE_DOUBLE:
               data.d[c] = op[0]->value.d[c0] * op[1]->value.d[c1];
               break;
            case GLSL_TYPE_UINT64:
               data.u64[c] = op[0]->value.u64[c0] * op[1]->value.u64[c1];
               break;
            case GLSL_TYPE_INT64:
               data.i64[c] = op[0]->value.i64[c0] * op[1]->value.i64[c1];
               break;
            default:
               unreachable("invalid type");
            }
         }
      } else {
         assert(op[0]->type->is_matrix() || op[1]->type->is_matrix());

         /* Multiply an N-by-M matrix with an M-by-P matrix.  Since either
          * matrix can be a GLSL vector, either N or P can be 1.
          *
          * For vec*mat, the vector is treated as a row vector.  This
          * means the vector is a 1-row x M-column matrix.
          *
          * For mat*vec, the vector is treated as a column vector.  Since
          * matrix_columns is 1 for vectors, this just works.
          */
         const unsigned n = op[0]->type->is_vector()
            ? 1 : op[0]->type->vector_elements;
         const unsigned m = op[1]->type->vector_elements;
         const unsigned p = op[1]->type->matrix_columns;
         for (unsigned j = 0; j < p; j++) {
            for (unsigned i = 0; i < n; i++) {
               for (unsigned k = 0; k < m; k++) {
                  if (op[0]->type->is_double())
                     data.d[i+n*j] += op[0]->value.d[i+n*k]*op[1]->value.d[k+m*j];
                  else
                     data.f[i+n*j] += op[0]->value.f[i+n*k]*op[1]->value.f[k+m*j];
               }
            }
         }
      }
      break;

   case ir_binop_mul_32x16:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c] * (uint16_t)op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c] * (int16_t)op[0]->value.i[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_div:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[1]->value.u[c1] == 0 ? 0 : op[0]->value.u[c0] / op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[1]->value.i[c1] == 0 ? 0 : op[0]->value.i[c0] / op[1]->value.i[c1];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c0] / op[1]->value.f[c1];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c0] / op[1]->value.d[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[1]->value.u64[c1] == 0 ? 0 : op[0]->value.u64[c0] / op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[1]->value.i64[c1] == 0 ? 0 : op[0]->value.i64[c0] / op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_mod:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[1]->value.u[c1] == 0 ? 0 : op[0]->value.u[c0] % op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[1]->value.i[c1] == 0 ? 0 : op[0]->value.i[c0] % op[1]->value.i[c1];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c0] - op[1]->value.f[c1] * floorf(op[0]->value.f[c0] / op[1]->value.f[c1]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c0] - op[1]->value.d[c1] * floor(op[0]->value.d[c0] / op[1]->value.d[c1]);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[1]->value.u64[c1] == 0 ? 0 : op[0]->value.u64[c0] % op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[1]->value.i64[c1] == 0 ? 0 : op[0]->value.i64[c0] % op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_less:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.b[c] = op[0]->value.u[c] < op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.b[c] = op[0]->value.i[c] < op[1]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.b[c] = op[0]->value.f[c] < op[1]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.b[c] = op[0]->value.d[c] < op[1]->value.d[c];
            break;
         case GLSL_TYPE_UINT64:
            data.b[c] = op[0]->value.u64[c] < op[1]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.b[c] = op[0]->value.i64[c] < op[1]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_gequal:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.b[c] = op[0]->value.u[c] >= op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.b[c] = op[0]->value.i[c] >= op[1]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.b[c] = op[0]->value.f[c] >= op[1]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.b[c] = op[0]->value.d[c] >= op[1]->value.d[c];
            break;
         case GLSL_TYPE_UINT64:
            data.b[c] = op[0]->value.u64[c] >= op[1]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.b[c] = op[0]->value.i64[c] >= op[1]->value.i64[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_equal:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.b[c] = op[0]->value.u[c] == op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.b[c] = op[0]->value.i[c] == op[1]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.b[c] = op[0]->value.f[c] == op[1]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.b[c] = op[0]->value.d[c] == op[1]->value.d[c];
            break;
         case GLSL_TYPE_UINT64:
            data.b[c] = op[0]->value.u64[c] == op[1]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.b[c] = op[0]->value.i64[c] == op[1]->value.i64[c];
            break;
         case GLSL_TYPE_BOOL:
            data.b[c] = op[0]->value.b[c] == op[1]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_nequal:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.b[c] = op[0]->value.u[c] != op[1]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.b[c] = op[0]->value.i[c] != op[1]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.b[c] = op[0]->value.f[c] != op[1]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.b[c] = op[0]->value.d[c] != op[1]->value.d[c];
            break;
         case GLSL_TYPE_UINT64:
            data.b[c] = op[0]->value.u64[c] != op[1]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.b[c] = op[0]->value.i64[c] != op[1]->value.i64[c];
            break;
         case GLSL_TYPE_BOOL:
            data.b[c] = op[0]->value.b[c] != op[1]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_all_equal:
      data.b[0] = op[0]->has_value(op[1]);
      break;

   case ir_binop_any_nequal:
      data.b[0] = !op[0]->has_value(op[1]);
      break;

   case ir_binop_lshift:
      assert(op[0]->type->base_type == GLSL_TYPE_UINT ||
             op[0]->type->base_type == GLSL_TYPE_INT ||
             op[0]->type->base_type == GLSL_TYPE_UINT64 ||
             op[0]->type->base_type == GLSL_TYPE_INT64);
      assert(op[1]->type->base_type == GLSL_TYPE_UINT ||
             op[1]->type->base_type == GLSL_TYPE_INT ||
             op[1]->type->base_type == GLSL_TYPE_UINT64 ||
             op[1]->type->base_type == GLSL_TYPE_INT64);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] << op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] << op[1]->value.i[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] << op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] << op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_rshift:
      assert(op[0]->type->base_type == GLSL_TYPE_UINT ||
             op[0]->type->base_type == GLSL_TYPE_INT ||
             op[0]->type->base_type == GLSL_TYPE_UINT64 ||
             op[0]->type->base_type == GLSL_TYPE_INT64);
      assert(op[1]->type->base_type == GLSL_TYPE_UINT ||
             op[1]->type->base_type == GLSL_TYPE_INT ||
             op[1]->type->base_type == GLSL_TYPE_UINT64 ||
             op[1]->type->base_type == GLSL_TYPE_INT64);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] >> op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] >> op[1]->value.i[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] >> op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] >> op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_bit_and:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] & op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] & op[1]->value.i[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] & op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] & op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_bit_xor:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] ^ op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] ^ op[1]->value.i[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] ^ op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] ^ op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_bit_or:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.u[c0] | op[1]->value.u[c1];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.i[c0] | op[1]->value.i[c1];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.u64[c0] | op[1]->value.u64[c1];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.i64[c0] | op[1]->value.i64[c1];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_logic_and:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.b[c] = op[0]->value.b[c] && op[1]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_logic_xor:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.b[c] = op[0]->value.b[c] != op[1]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_logic_or:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_BOOL:
            data.b[c] = op[0]->value.b[c] || op[1]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_dot:
      switch (op[0]->type->base_type) {
      case GLSL_TYPE_FLOAT:
         data.f[0] = dot_f(op[0], op[1]);
         break;
      case GLSL_TYPE_DOUBLE:
         data.d[0] = dot_d(op[0], op[1]);
         break;
      default:
         unreachable("invalid type");
      }
      break;

   case ir_binop_min:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = MIN2(op[0]->value.u[c0], op[1]->value.u[c1]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = MIN2(op[0]->value.i[c0], op[1]->value.i[c1]);
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = MIN2(op[0]->value.f[c0], op[1]->value.f[c1]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = MIN2(op[0]->value.d[c0], op[1]->value.d[c1]);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = MIN2(op[0]->value.u64[c0], op[1]->value.u64[c1]);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = MIN2(op[0]->value.i64[c0], op[1]->value.i64[c1]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_max:
      assert(op[0]->type == op[1]->type || op0_scalar || op1_scalar);
      for (unsigned c = 0, c0 = 0, c1 = 0;
           c < components;
           c0 += c0_inc, c1 += c1_inc, c++) {

         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = MAX2(op[0]->value.u[c0], op[1]->value.u[c1]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = MAX2(op[0]->value.i[c0], op[1]->value.i[c1]);
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = MAX2(op[0]->value.f[c0], op[1]->value.f[c1]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = MAX2(op[0]->value.d[c0], op[1]->value.d[c1]);
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = MAX2(op[0]->value.u64[c0], op[1]->value.u64[c1]);
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = MAX2(op[0]->value.i64[c0], op[1]->value.i64[c1]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_pow:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = powf(op[0]->value.f[c], op[1]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_ldexp:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = ldexpf_flush_subnormal(op[0]->value.f[c], op[1]->value.i[c]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = ldexp_flush_subnormal(op[0]->value.d[c], op[1]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_binop_vector_extract: {
      const int c = CLAMP(op[1]->value.i[0], 0,
                          (int) op[0]->type->vector_elements - 1);

      switch (op[0]->type->base_type) {
      case GLSL_TYPE_UINT:
         data.u[0] = op[0]->value.u[c];
         break;
      case GLSL_TYPE_INT:
         data.i[0] = op[0]->value.i[c];
         break;
      case GLSL_TYPE_FLOAT:
         data.f[0] = op[0]->value.f[c];
         break;
      case GLSL_TYPE_DOUBLE:
         data.d[0] = op[0]->value.d[c];
         break;
      case GLSL_TYPE_UINT64:
         data.u64[0] = op[0]->value.u64[c];
         break;
      case GLSL_TYPE_INT64:
         data.i64[0] = op[0]->value.i64[c];
         break;
      case GLSL_TYPE_BOOL:
         data.b[0] = op[0]->value.b[c];
         break;
      default:
         unreachable("invalid type");
      }
      break;
   }

   case ir_binop_atan2:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = atan2(op[0]->value.f[c], op[1]->value.f[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_triop_fma:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c] * op[1]->value.f[c] + op[2]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c] * op[1]->value.d[c] + op[2]->value.d[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_triop_lrp: {
      assert(op[0]->type->is_float() || op[0]->type->is_double());
      assert(op[1]->type->is_float() || op[1]->type->is_double());
      assert(op[2]->type->is_float() || op[2]->type->is_double());

      unsigned c2_inc = op[2]->type->is_scalar() ? 0 : 1;
      for (unsigned c = 0, c2 = 0; c < components; c2 += c2_inc, c++) {
         switch (this->type->base_type) {
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.f[c] * (1.0f - op[2]->value.f[c2]) + (op[1]->value.f[c] * op[2]->value.f[c2]);
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.d[c] * (1.0 - op[2]->value.d[c2]) + (op[1]->value.d[c] * op[2]->value.d[c2]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;
   }

   case ir_triop_csel:
      for (unsigned c = 0; c < components; c++) {
         switch (this->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[0]->value.b[c] ? op[1]->value.u[c] : op[2]->value.u[c];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[0]->value.b[c] ? op[1]->value.i[c] : op[2]->value.i[c];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[0]->value.b[c] ? op[1]->value.f[c] : op[2]->value.f[c];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[0]->value.b[c] ? op[1]->value.d[c] : op[2]->value.d[c];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[0]->value.b[c] ? op[1]->value.u64[c] : op[2]->value.u64[c];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[0]->value.b[c] ? op[1]->value.i64[c] : op[2]->value.i64[c];
            break;
         case GLSL_TYPE_BOOL:
            data.b[c] = op[0]->value.b[c] ? op[1]->value.b[c] : op[2]->value.b[c];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_triop_bitfield_extract:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.i[c] = bitfield_extract_uint(op[0]->value.u[c], op[1]->value.i[c], op[2]->value.i[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = bitfield_extract_int(op[0]->value.i[c], op[1]->value.i[c], op[2]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_triop_vector_insert: {
      const unsigned idx = op[2]->value.u[0];

      memcpy(&data, &op[0]->value, sizeof(data));

      switch (this->type->base_type) {
      case GLSL_TYPE_UINT:
         data.u[idx] = op[1]->value.u[0];
         break;
      case GLSL_TYPE_INT:
         data.i[idx] = op[1]->value.i[0];
         break;
      case GLSL_TYPE_FLOAT:
         data.f[idx] = op[1]->value.f[0];
         break;
      case GLSL_TYPE_DOUBLE:
         data.d[idx] = op[1]->value.d[0];
         break;
      case GLSL_TYPE_UINT64:
         data.u64[idx] = op[1]->value.u64[0];
         break;
      case GLSL_TYPE_INT64:
         data.i64[idx] = op[1]->value.i64[0];
         break;
      case GLSL_TYPE_BOOL:
         data.b[idx] = op[1]->value.b[0];
         break;
      default:
         unreachable("invalid type");
      }
      break;
   }

   case ir_quadop_bitfield_insert:
      for (unsigned c = 0; c < op[0]->type->components(); c++) {
         switch (op[0]->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = bitfield_insert(op[0]->value.u[c], op[1]->value.u[c], op[2]->value.i[c], op[3]->value.i[c]);
            break;
         case GLSL_TYPE_INT:
            data.i[c] = bitfield_insert(op[0]->value.i[c], op[1]->value.i[c], op[2]->value.i[c], op[3]->value.i[c]);
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   case ir_quadop_vector:
      for (unsigned c = 0; c < this->type->vector_elements; c++) {
         switch (this->type->base_type) {
         case GLSL_TYPE_UINT:
            data.u[c] = op[c]->value.u[0];
            break;
         case GLSL_TYPE_INT:
            data.i[c] = op[c]->value.i[0];
            break;
         case GLSL_TYPE_FLOAT:
            data.f[c] = op[c]->value.f[0];
            break;
         case GLSL_TYPE_DOUBLE:
            data.d[c] = op[c]->value.d[0];
            break;
         case GLSL_TYPE_UINT64:
            data.u64[c] = op[c]->value.u64[0];
            break;
         case GLSL_TYPE_INT64:
            data.i64[c] = op[c]->value.i64[0];
            break;
         case GLSL_TYPE_BOOL:
            data.b[c] = op[c]->value.b[0];
            break;
         default:
            unreachable("invalid type");
         }
      }
      break;

   default:
      /* FINISHME: Should handle all expression types. */
      return NULL;
   }

