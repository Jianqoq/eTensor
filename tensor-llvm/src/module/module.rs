use std::{ mem::MaybeUninit, path::Path };

use llvm_sys::{
    core::{
        LLVMAddFunction,
        LLVMAddGlobalInAddressSpace,
        LLVMGetNamedFunction,
        LLVMModuleCreateWithNameInContext,
        LLVMPrintModuleToFile,
    },
    LLVMModule,
};
use crate::{
    context::context::Context,
    types::{ general_types::GeneralType, values::{ FunctionValue, StructValue } },
    utils::to_c_str,
    FunctionType,
};

pub struct Module {
    pub(crate) module: *mut LLVMModule,
}

impl Module {
    pub fn inner(&self) -> *mut LLVMModule {
        self.module
    }

    pub fn new(module_name: &str, ctx: &Context) -> Self {
        let module = unsafe {
            LLVMModuleCreateWithNameInContext(to_c_str(module_name).as_ptr(), ctx.inner())
        };
        Module { module }
    }

    pub fn get_function(&self, fn_type: FunctionType, name: &str) -> Option<FunctionValue> {
        let c_string = to_c_str(name);
        let fn_value = unsafe { LLVMGetNamedFunction(self.module, c_string.as_ptr()) };
        if fn_value.is_null() {
            None
        } else {
            Some(
                FunctionValue::new(
                    fn_value,
                    fn_type.ret_type().as_ref().clone(),
                    fn_type.param_types(),
                    fn_type.param_count()
                )
            )
        }
    }

    pub fn add_function(&self, fn_type: FunctionType, name: &str) -> FunctionValue {
        let c_string = to_c_str(name);
        let fn_val = unsafe { LLVMAddFunction(self.module, c_string.as_ptr(), fn_type.inner()) };
        if fn_val.is_null() {
            panic!("{}", &format!("function {} not added", name));
        }
        let fn_value = FunctionValue::new(
            fn_val,
            fn_type.ret_type().as_ref().clone(),
            fn_type.param_types(),
            fn_type.param_count()
        );
        fn_value
    }

    pub fn add_global_struct(
        &self,
        ty: &GeneralType,
        address_space: u32,
        name: &str
    ) -> StructValue {
        let c_string = to_c_str(name);
        let global = unsafe {
            LLVMAddGlobalInAddressSpace(self.module, ty.inner(), c_string.as_ptr(), address_space)
        };
        if global.is_null() {
            panic!("{}", &format!("global {} not added", name));
        }
        StructValue {
            value: global,
        }
    }

    pub fn print_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path_str = path.as_ref().to_str().expect("Did not find a valid Unicode path string");
        let path = to_c_str(path_str);
        let mut err_string = MaybeUninit::uninit();
        let return_code = unsafe {
            LLVMPrintModuleToFile(self.module, path.as_ptr(), err_string.as_mut_ptr())
        };

        if return_code == 1 {
            panic!("Failed to write module to file");
        }

        Ok(())
    }
}
