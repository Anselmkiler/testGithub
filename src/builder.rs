use llvm_sys::core::{LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildArrayAlloca, LLVMBuildArrayMalloc, LLVMBuildBr, LLVMBuildCall, LLVMBuildCast, LLVMBuildCondBr, LLVMBuildExtractValue, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFence, LLVMBuildFMul, LLVMBuildFNeg, LLVMBuildFree, LLVMBuildFSub, LLVMBuildGEP, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildIsNotNull, LLVMBuildIsNull, LLVMBuildLoad, LLVMBuildMalloc, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr, LLVMBuildPhi, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildUnreachable, LLVMBuildXor, LLVMDisposeBuilder, LLVMGetElementType, LLVMGetInsertBlock, LLVMGetReturnType, LLVMGetTypeKind, LLVMInsertIntoBuilder, LLVMPositionBuilderAtEnd, LLVMTypeOf, LLVMSetTailCall, LLVMBuildExtractElement, LLVMBuildInsertElement, LLVMBuildIntToPtr, LLVMBuildPtrToInt, LLVMInsertIntoBuilderWithName, LLVMClearInsertionPosition};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

use basic_block::BasicBlock;
use values::{AggregateValue, AsValueRef, BasicValue, BasicValueEnum, PhiValue, FunctionValue, FloatValue, IntValue, PointerValue, VectorValue, InstructionValue};
use types::{AsTypeRef, AnyType, BasicType, PointerType, IntType};

use std::ffi::CString;

pub struct Builder {
    builder: LLVMBuilderRef,
}

impl Builder {
    pub(crate) fn new(builder: LLVMBuilderRef) -> Self {
        assert!(!builder.is_null());

        Builder {
            builder: builder
        }
    }

    pub fn build_return(&self, value: Option<&BasicValue>) -> InstructionValue {
        // let value = unsafe {
        //     value.map_or(LLVMBuildRetVoid(self.builder), |value| LLVMBuildRet(self.builder, value.value))
        // };

        let value = unsafe {
            match value {
                Some(v) => LLVMBuildRet(self.builder, v.as_value_ref()),
                None => LLVMBuildRetVoid(self.builder),
            }
        };

        InstructionValue::new(value)
    }

    pub fn build_call(&self, function: &FunctionValue, args: &[&BasicValue], name: &str, tail_call: bool) -> BasicValueEnum {
        // LLVM gets upset when void calls are named because they don't return anything
        let name = unsafe {
            match LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(function.as_value_ref())))) {
                LLVMTypeKind::LLVMVoidTypeKind => "",
                _ => name,
            }
        };

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let mut args: Vec<LLVMValueRef> = args.iter()
                                              .map(|val| val.as_value_ref())
                                              .collect();
        let value = unsafe {
            LLVMBuildCall(self.builder, function.as_value_ref(), args.as_mut_ptr(), args.len() as u32, c_string.as_ptr())
        };

        if tail_call {
            unsafe {
                LLVMSetTailCall(value, true as i32)
            }
        }

        BasicValueEnum::new(value)
    }

    pub fn build_gep(&self, ptr: &PointerValue, ordered_indexes: &[&IntValue], name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = unsafe {
            LLVMBuildGEP(self.builder, ptr.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_phi(&self, type_: &AnyType, name: &str) -> PhiValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PhiValue::new(value)
    }

    pub fn build_store(&self, ptr: &PointerValue, value: &BasicValue) -> InstructionValue {
        let value = unsafe {
            LLVMBuildStore(self.builder, value.as_value_ref(), ptr.as_value_ref())
        };

        InstructionValue::new(value)
    }

    pub fn build_load(&self, ptr: &PointerValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildLoad(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_stack_allocation(&self, type_: &BasicType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_heap_allocation(&self, type_: &BasicType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODO: Rename to "build_heap_allocated_array" + stack version?
    // REVIEW: Is this still a PointerValue (as opposed to an ArrayValue?)
    pub fn build_array_heap_allocation(&self, type_: &BasicType, size: &IntValue, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, type_.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // REVIEW: Is this still a PointerValue (as opposed to an ArrayValue?)
    pub fn build_stack_allocated_array(&self, type_: &BasicType, size: &IntValue, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, type_.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_free(&self, ptr: &PointerValue) -> InstructionValue {
        let val = unsafe {
            LLVMBuildFree(self.builder, ptr.as_value_ref())
        };

        InstructionValue::new(val)
    }

    pub fn insert_instruction(&self, value: &InstructionValue) {
        unsafe {
            LLVMInsertIntoBuilder(self.builder, value.as_value_ref());
        }
    }

    pub fn insert_instruction_with_name(&self, instruction: &InstructionValue, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMInsertIntoBuilderWithName(self.builder, instruction.as_value_ref(), c_string.as_ptr())
        }
    }

    pub fn get_insert_block(&self) -> BasicBlock {
        let bb = unsafe {
            LLVMGetInsertBlock(self.builder)
        };

        BasicBlock::new(bb)
    }

    pub fn build_int_div(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        // TODO: Support signed, possibly as metadata on IntValue?
        let value = unsafe {
            LLVMBuildUDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_float_div(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_int_add(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_float_add(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_xor(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildXor(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_and(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAnd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_or(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildOr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_int_sub(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_float_sub(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_int_mul(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_float_mul(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_cast(&self, op: LLVMOpcode, from_value: &BasicValue, to_type: &BasicType, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op, from_value.as_value_ref(), to_type.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_pointer_cast(&self, from: &PointerValue, to: &PointerType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPointerCast(self.builder, from.as_value_ref(), to.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_int_compare(&self, op: LLVMIntPredicate, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildICmp(self.builder, op, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_float_compare(&self, op: LLVMRealPredicate, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFCmp(self.builder, op, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_unconditional_branch(&self, destination_block: &BasicBlock) -> InstructionValue {
        let value = unsafe {
            LLVMBuildBr(self.builder, destination_block.basic_block)
        };

        InstructionValue::new(value)
    }

    pub fn build_conditional_branch(&self, comparison: &IntValue, then_block: &BasicBlock, else_block: &BasicBlock) -> InstructionValue {
        let value = unsafe {
            LLVMBuildCondBr(self.builder, comparison.as_value_ref(), then_block.basic_block, else_block.basic_block)
        };

        InstructionValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_int_neg(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Untested
    pub fn build_float_neg(&self, value: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_not(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNot(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn position_at_end(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, basic_block.basic_block);
        }
    }

    // REVIEW: How does LLVM treat out of bound index? Maybe we should return an Option?
    // or is that only in bounds GEP
    pub fn build_extract_value(&self, value: &AggregateValue, index: u32, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractValue(self.builder, value.as_value_ref(), index, c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_insert_value(&self, value: &BasicValue, ptr: &PointerValue, index: u32, name: &str) -> InstructionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertValue(self.builder, value.as_value_ref(), ptr.as_value_ref(), index, c_string.as_ptr())
        };

        InstructionValue::new(value)
    }

    pub fn build_extract_element(&self, vector: &VectorValue, index: &IntValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractElement(self.builder, vector.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_insert_element(&self, vector: &VectorValue, element: &BasicValue, index: &IntValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertElement(self.builder, vector.as_value_ref(), element.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_unreachable(&self) -> InstructionValue {
        let val = unsafe {
            LLVMBuildUnreachable(self.builder)
        };

        InstructionValue::new(val)
    }

    // REVIEW: Not sure if this should return InstructionValue or an actual value
    // TODO: Better name for num?
    pub fn build_fence(&self, atmoic_ordering: LLVMAtomicOrdering, num: i32, name: &str) -> InstructionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildFence(self.builder, atmoic_ordering, num, c_string.as_ptr())
        };

        InstructionValue::new(val)
    }

    pub fn build_is_null(&self, ptr: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(val)
    }

    pub fn build_is_not_null(&self, ptr: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNotNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(val)
    }

    pub fn build_int_to_ptr(&self, int: &IntValue, ptr_type: &PointerType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntToPtr(self.builder, int.as_value_ref(), ptr_type.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_ptr_to_int(&self, ptr: &PointerValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPtrToInt(self.builder, ptr.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn clear_insertion_position(&self) {
        unsafe {
            LLVMClearInsertionPosition(self.builder)
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}

#[test]
fn test_build_call() {
    use context::Context;

    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let pi = f32_type.const_float(3.14);

    builder.build_return(Some(&pi));

    let function2 = module.add_function("wrapper", &fn_type, None);
    let basic_block2 = context.append_basic_block(&function2, "entry");

    builder.position_at_end(&basic_block2);

    let pi2 = builder.build_call(&function, &[], "get_pi", false);

    builder.build_return(Some(&pi2));
}

#[test]
fn test_instructions() {
    use context::Context;
    use values::InstructionOpcode::*;

    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(0);
    let fn_type = void_type.fn_type(&[&f32_ptr_type], false);

    let function = module.add_function("free_f32", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();

    let f32_val = f32_type.const_float(3.14);

    let store_instruction = builder.build_store(&arg1, &f32_val);
    let ptr_val = builder.build_ptr_to_int(&arg1, &i64_type, "ptr_val");
    let ptr = builder.build_int_to_ptr(&ptr_val, &f32_ptr_type, "ptr");
    let free_instruction = builder.build_free(&arg1);
    let return_instruction = builder.build_return(None);

    assert_eq!(store_instruction.get_opcode(), Store);
    assert_eq!(ptr_val.as_instruction().unwrap().get_opcode(), PtrToInt);
    assert_eq!(ptr.as_instruction().unwrap().get_opcode(), IntToPtr);
    assert_eq!(free_instruction.get_opcode(), Call);
    assert_eq!(return_instruction.get_opcode(), Return);
}
