use std::{ fmt::Display, sync::Arc };

use tensor_llvm::{ context::context::Context, types::general_types::GeneralType, BoolType };
use tensor_types::{ dtype::Dtype, type_promote::{ FloatOut, NormalOut } };

#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum PrimitiveType {
    Dtype(Dtype),
    Tuple(Tuple),
    Array(Array),
    Ptr(Ptr),
    Str,
    Void,
}
#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tuple {
    pub inner: Arc<Vec<PrimitiveType>>,
}
#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Array {
    pub inner: Arc<PrimitiveType>,
    pub size: i64,
}
#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Ptr {
    pub inner: Arc<PrimitiveType>,
}

impl PrimitiveType {
    pub fn to_llvm_type(&self, ctx: &Context) -> GeneralType {
        match self {
            PrimitiveType::Dtype(dtype) => {
                match dtype {
                    Dtype::Bool => GeneralType::Bool(ctx.bool_type()),
                    Dtype::I8 => GeneralType::I8(ctx.i8_type()),
                    Dtype::U8 => GeneralType::U8(ctx.u8_type()),
                    Dtype::I16 => GeneralType::I16(ctx.i16_type()),
                    Dtype::U16 => GeneralType::U16(ctx.u16_type()),
                    Dtype::I32 => GeneralType::I32(ctx.i32_type()),
                    Dtype::U32 => GeneralType::U32(ctx.u32_type()),
                    Dtype::I64 => GeneralType::I64(ctx.i64_type()),
                    Dtype::U64 => GeneralType::U64(ctx.u64_type()),
                    Dtype::F16 => GeneralType::F16(ctx.f16_type()),
                    Dtype::F32 => GeneralType::F32(ctx.f32_type()),
                    Dtype::F64 => GeneralType::F64(ctx.f64_type()),
                    Dtype::Isize => GeneralType::Isize(ctx.isize_type()),
                    _ => unimplemented!(),
                }
            }
            PrimitiveType::Tuple(tuple) => {
                let mut inner = Vec::new();
                for t in tuple.inner.iter() {
                    inner.push(t.to_llvm_type(ctx));
                }
                let struct_type = ctx.struct_type(&inner, false);
                GeneralType::Struct(struct_type)
            }
            _ => unimplemented!(),
        }
    }

    pub fn dtype(&self) -> Dtype {
        match self {
            PrimitiveType::Dtype(dtype) => *dtype,
            _ => unimplemented!(),
        }
    }
    pub fn _add(&self, other: &Self) -> Self {
        match (self, other) {
            (PrimitiveType::Dtype(lhs), PrimitiveType::Dtype(rhs)) => {
                PrimitiveType::Dtype(lhs._add(*rhs))
            }
            _ => unimplemented!(),
        }
    }
    pub fn _mul(&self, other: &Self) -> Self {
        match (self, other) {
            (PrimitiveType::Dtype(lhs), PrimitiveType::Dtype(rhs)) => {
                PrimitiveType::Dtype(lhs._mul(*rhs))
            }
            _ => unimplemented!(),
        }
    }
    pub fn _sub(&self, other: &Self) -> Self {
        match (self, other) {
            (PrimitiveType::Dtype(lhs), PrimitiveType::Dtype(rhs)) => {
                PrimitiveType::Dtype(lhs._sub(*rhs))
            }
            _ => unimplemented!(),
        }
    }
    pub fn _div(&self, other: &Self) -> Self {
        match (self, other) {
            (PrimitiveType::Dtype(lhs), PrimitiveType::Dtype(rhs)) => {
                PrimitiveType::Dtype(lhs._div(*rhs))
            }
            _ => unimplemented!(),
        }
    }
    pub fn _sin(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._sin()),
            _ => unimplemented!(),
        }
    }
    pub fn _cos(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._cos()),
            _ => unimplemented!(),
        }
    }
    pub fn _tan(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._tan()),
            _ => unimplemented!(),
        }
    }
    pub fn _asin(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._asin()),
            _ => unimplemented!(),
        }
    }
    pub fn _acos(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._acos()),
            _ => unimplemented!(),
        }
    }
    pub fn _atan(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._atan()),
            _ => unimplemented!(),
        }
    }
    pub fn _exp(&self) -> Self {
        match self {
            PrimitiveType::Dtype(dtype) => PrimitiveType::Dtype(dtype._exp()),
            _ => unimplemented!(),
        }
    }
    pub fn _log(&self, base: &Self) -> Self {
        match (self, base) {
            (PrimitiveType::Dtype(lhs), PrimitiveType::Dtype(rhs)) => {
                PrimitiveType::Dtype(lhs._log(*rhs))
            }
            _ => unimplemented!(),
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Dtype(dtype) => write!(f, "{}", dtype),
            PrimitiveType::Tuple(tuple) => {
                if tuple.inner.is_empty() {
                    return write!(f, "()");
                }
                if tuple.inner.len() == 1 {
                    return write!(f, "({}, )", tuple.inner[0]);
                }
                write!(f, "({}", tuple.inner[0])?;
                for i in 1..tuple.inner.len() {
                    write!(f, ", {}", tuple.inner[i])?;
                }
                write!(f, ")")
            }
            PrimitiveType::Array(arr) => write!(f, "[{}; {}]", arr.inner, arr.size),
            PrimitiveType::Ptr(ptr) => write!(f, "*{}", ptr.inner),
            PrimitiveType::Str => write!(f, "str"),
            PrimitiveType::Void => write!(f, "void"),
        }
    }
}
