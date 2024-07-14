use llvm_sys::{
    core::{
        LLVMAppendBasicBlockInContext,
        LLVMContextCreate,
        LLVMDoubleTypeInContext,
        LLVMFloatTypeInContext,
        LLVMHalfTypeInContext,
        LLVMInt16TypeInContext,
        LLVMInt1TypeInContext,
        LLVMInt32TypeInContext,
        LLVMInt64TypeInContext,
        LLVMInt8TypeInContext,
        LLVMPointerType,
        LLVMStructTypeInContext,
        LLVMVoidTypeInContext,
    },
    execution_engine::LLVMLinkInMCJIT,
    target::{
        LLVM_InitializeNativeAsmParser,
        LLVM_InitializeNativeAsmPrinter,
        LLVM_InitializeNativeDisassembler,
        LLVM_InitializeNativeTarget,
    },
    LLVMContext,
};
use crate::{
    types::{
        block::BasicBlock,
        general_types::GeneralType,
        ptr_type::StrPtrType,
        values::FunctionValue,
    },
    utils::to_c_str,
    BoolType,
    F16Type,
    F32Type,
    F64Type,
    I16Type,
    I32Type,
    I64Type,
    I8Type,
    IsizeType,
    StructType,
    U16Type,
    U32Type,
    U64Type,
    U8Type,
    VoidType,
};

pub struct Context {
    pub(crate) context: *mut LLVMContext,
}

impl Context {
    pub fn new() -> Self {
        let context = unsafe { LLVMContextCreate() };
        unsafe {
            LLVMLinkInMCJIT();
            let code = LLVM_InitializeNativeTarget();
            if code == 1 {
                panic!("Failed to initialize native target");
            }
            let code = LLVM_InitializeNativeAsmPrinter();
            if code == 1 {
                panic!("Failed to initialize native asm printer");
            }
            let code = LLVM_InitializeNativeAsmParser();
            if code == 1 {
                panic!("Failed to initialize native asm parser");
            }
            let node = LLVM_InitializeNativeDisassembler();
            if node == 1 {
                panic!("Failed to initialize native asm printer");
            }
        }
        Context { context }
    }
    pub(crate) fn inner(&self) -> *mut LLVMContext {
        self.context
    }
    pub fn append_basic_block(&self, function: &FunctionValue, name: &str) -> BasicBlock {
        let name = to_c_str(name);
        let block = unsafe {
            LLVMAppendBasicBlockInContext(self.context, function.value(), name.as_ptr())
        };
        BasicBlock::from(block)
    }

    pub fn str_ptr_type(&self) -> StrPtrType {
        StrPtrType::from(unsafe { LLVMPointerType(LLVMInt8TypeInContext(self.context), 0) })
    }

    pub fn bool_type(&self) -> BoolType {
        BoolType::from(unsafe { LLVMInt1TypeInContext(self.context) })
    }

    pub fn i8_type(&self) -> I8Type {
        I8Type::from(unsafe { LLVMInt8TypeInContext(self.context) })
    }

    pub fn i16_type(&self) -> I16Type {
        I16Type::from(unsafe { LLVMInt16TypeInContext(self.context) })
    }

    pub fn i32_type(&self) -> I32Type {
        I32Type::from(unsafe { LLVMInt32TypeInContext(self.context) })
    }

    pub fn i64_type(&self) -> I64Type {
        I64Type::from(unsafe { LLVMInt64TypeInContext(self.context) })
    }

    pub fn u8_type(&self) -> U8Type {
        U8Type::from(unsafe { LLVMInt8TypeInContext(self.context) })
    }

    pub fn u16_type(&self) -> U16Type {
        U16Type::from(unsafe { LLVMInt16TypeInContext(self.context) })
    }

    pub fn u32_type(&self) -> U32Type {
        U32Type::from(unsafe { LLVMInt32TypeInContext(self.context) })
    }

    pub fn u64_type(&self) -> U64Type {
        U64Type::from(unsafe { LLVMInt64TypeInContext(self.context) })
    }
    pub fn usize_type(&self) -> U64Type {
        match std::mem::size_of::<usize>() {
            4 => U64Type::from(unsafe { LLVMInt32TypeInContext(self.context) }),
            8 => U64Type::from(unsafe { LLVMInt64TypeInContext(self.context) }),
            _ => panic!("Unsupported usize size"),
        }
    }

    pub fn isize_type(&self) -> IsizeType {
        match std::mem::size_of::<isize>() {
            4 => IsizeType::from(unsafe { LLVMInt32TypeInContext(self.context) }),
            8 => IsizeType::from(unsafe { LLVMInt64TypeInContext(self.context) }),
            _ => panic!("Unsupported isize size"),
        }
    }

    pub fn f16_type(&self) -> F16Type {
        F16Type::from(unsafe { LLVMHalfTypeInContext(self.context) })
    }

    pub fn f32_type(&self) -> F32Type {
        F32Type::from(unsafe { LLVMFloatTypeInContext(self.context) })
    }

    pub fn f64_type(&self) -> F64Type {
        F64Type::from(unsafe { LLVMDoubleTypeInContext(self.context) })
    }

    pub fn void_type(&self) -> VoidType {
        VoidType::from(unsafe { LLVMVoidTypeInContext(self.context) })
    }

    pub fn struct_type(&self, types: &[GeneralType], packed: bool) -> StructType {
        let mut types = types
            .iter()
            .map(|t| t.inner())
            .collect::<Vec<_>>();
        let struct_type = unsafe {
            LLVMStructTypeInContext(
                self.context,
                types.as_mut_ptr(),
                types.len() as u32,
                packed as i32
            )
        };
        StructType::from(struct_type)
    }
}
