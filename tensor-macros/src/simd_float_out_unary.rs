use proc_macro::TokenStream;
use quote::quote;
use crate::type_utils::{ type_simd_lanes, SimdType, TypeInfo };
use crate::type_utils::type_simd_is_arr;
use crate::type_utils::Type;
use proc_macro2::Ident;

pub fn impl_float_out_unary() -> TokenStream {
    let mut ret = proc_macro2::TokenStream::new();

    let types = [
        (format!("boolx{}", type_simd_lanes("bool")), "bool"),
        (format!("bf16x{}", type_simd_lanes("bf16")), "bf16"),
        (format!("f16x{}", type_simd_lanes("f16")), "f16"),
        (format!("f32x{}", type_simd_lanes("f32")), "f32"),
        (format!("f64x{}", type_simd_lanes("f64")), "f64"),
        (format!("i8x{}", type_simd_lanes("i8")), "i8"),
        (format!("i16x{}", type_simd_lanes("i16")), "i16"),
        (format!("i32x{}", type_simd_lanes("i32")), "i32"),
        (format!("i64x{}", type_simd_lanes("i64")), "i64"),
        (format!("u8x{}", type_simd_lanes("u8")), "u8"),
        (format!("u16x{}", type_simd_lanes("u16")), "u16"),
        (format!("u32x{}", type_simd_lanes("u32")), "u32"),
        (format!("u64x{}", type_simd_lanes("u64")), "u64"),
        (format!("isizex{}", type_simd_lanes("isize")), "isize"),
        (format!("usizex{}", type_simd_lanes("usize")), "usize"),
    ];

    for (_, lhs) in types.iter() {
        let lhs_type = TypeInfo::new(lhs);
        let res_type = lhs_type.infer_float_res_type_uary();
        let lhs_lanes = type_simd_lanes(lhs);
        let lhs_simd: SimdType = (*lhs).into();
        let res_simd_ty = Ident::new(
            &format!("{}x{}", res_type.to_string(), type_simd_lanes(&res_type.to_string())),
            proc_macro2::Span::call_site()
        );
        let res_lanes = type_simd_lanes(&res_type.to_string());
        if res_lanes != lhs_lanes {
            let res = unreachable_impl(lhs_simd, res_type, res_simd_ty.clone());
            ret.extend(res);
            continue;
        }
        if type_simd_is_arr(lhs) || type_simd_is_arr(&res_type.to_string()) {
            let sin = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_sin", proc_macro2::Span::call_site())
            );
            let cos = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_cos", proc_macro2::Span::call_site())
            );
            let tan = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_tan", proc_macro2::Span::call_site())
            );
            let asin = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_asin", proc_macro2::Span::call_site())
            );
            let acos = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_acos", proc_macro2::Span::call_site())
            );
            let atan = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_atan", proc_macro2::Span::call_site())
            );
            let sinh = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_sinh", proc_macro2::Span::call_site())
            );
            let cosh = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_cosh", proc_macro2::Span::call_site())
            );
            let tanh = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_tanh", proc_macro2::Span::call_site())
            );
            let asinh = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_asinh", proc_macro2::Span::call_site())
            );
            let acosh = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_acosh", proc_macro2::Span::call_site())
            );
            let atanh = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_atanh", proc_macro2::Span::call_site())
            );
            let recip = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_recip", proc_macro2::Span::call_site())
            );
            let erf = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_erf", proc_macro2::Span::call_site())
            );
            let sigmoid = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_sigmoid", proc_macro2::Span::call_site())
            );
            let ln = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_ln", proc_macro2::Span::call_site())
            );
            let exp = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_exp", proc_macro2::Span::call_site())
            );
            let exp2 = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_exp2", proc_macro2::Span::call_site())
            );
            let log2 = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_log2", proc_macro2::Span::call_site())
            );
            let log10 = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_log10", proc_macro2::Span::call_site())
            );
            let sqrt = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_sqrt", proc_macro2::Span::call_site())
            );
            let relu = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_relu", proc_macro2::Span::call_site())
            );
            let gelu = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_gelu", proc_macro2::Span::call_site())
            );
            let relu6 = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_relu6", proc_macro2::Span::call_site())
            );
            let _hard_swish = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_hard_swish", proc_macro2::Span::call_site())
            );
            let soft_plus = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_softplus", proc_macro2::Span::call_site())
            );
            let softsign = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_softsign", proc_macro2::Span::call_site())
            );
            let mish = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_mish", proc_macro2::Span::call_site())
            );
            let cbrt = gen_func_arr(
                lhs_lanes,
                res_type,
                res_simd_ty.clone(),
                Ident::new("_cbrt", proc_macro2::Span::call_site())
            );
            let celu = {
                let unroll = (0..lhs_lanes as usize).map(|i| {
                    quote! {
                        arr[#i] = self_arr[#i]._celu(alpha);
                    }
                });
                quote! {
                    fn _celu(self, alpha: Self::Base) -> Self::Output {
                        paste::paste! {
                            let mut arr = [#res_type::ZERO; #lhs_lanes as usize];
                            let self_arr = self.0;
                            #(#unroll)*
                            #res_simd_ty::#res_simd_ty(arr.into())
                        }
                    }
                }
            };
            let selu = {
                let unroll = (0..lhs_lanes as usize).map(|i| {
                    quote! {
                        arr[#i] = self_arr[#i]._selu(alpha, scale);
                    }
                });
                quote! {
                    fn _selu(self, alpha: Self::Base, scale: Self::Base) -> Self::Output {
                        paste::paste! {
                            let mut arr = [#res_type::ZERO; #lhs_lanes as usize];
                            let self_arr = self.0;
                            #(#unroll)*
                            #res_simd_ty::#res_simd_ty(arr.into())
                        }
                    }
                }
            };
            let elu = {
                let unroll = (0..lhs_lanes as usize).map(|i| {
                    quote! {
                        arr[#i] = self_arr[#i]._elu(alpha);
                    }
                });
                quote! {
                    fn _elu(self, alpha: Self::Base) -> Self::Output {
                        paste::paste! {
                            let mut arr = [#res_type::ZERO; #lhs_lanes as usize];
                            let self_arr = self.0;
                            #(#unroll)*
                            #res_simd_ty::#res_simd_ty(arr.into())
                        }
                    }
                }
            };
            let leaky_relu = {
                let unroll = (0..lhs_lanes as usize).map(|i| {
                    quote! {
                        arr[#i] = self_arr[#i]._leaky_relu(alpha);
                    }
                });
                quote! {
                    fn _leaky_relu(self, alpha: Self::Base) -> Self::Output {
                        paste::paste! {
                            let mut arr = [#res_type::ZERO; #lhs_lanes as usize];
                            let self_arr = self.0;
                            #(#unroll)*
                            #res_simd_ty::#res_simd_ty(arr.into())
                        }
                    }
                }
            };
            let hard_sigmoid = {
                let unroll = (0..lhs_lanes as usize).map(|i| {
                    quote! {
                        arr[#i] = self_arr[#i]._hard_sigmoid();
                    }
                });
                quote! {
                    fn _hard_sigmoid(self) -> Self::Output {
                        paste::paste! {
                            let mut arr = [#res_type::ZERO; #lhs_lanes as usize];
                            let self_arr = self.0;
                            #(#unroll)*
                            #res_simd_ty::#res_simd_ty(arr.into())
                        }
                    }
                }
            };
            let res =
                quote! {
                impl FloatOutUnary for #lhs_simd {
                    type Output = #res_simd_ty::#res_simd_ty;
                    type Base = #res_type;
                    #exp #exp2 #ln #log2 #log10 #sqrt #sin #cos #tan #asin #acos #atan #sinh #cosh #tanh #asinh #acosh
                    #atanh #recip #erf #sigmoid #relu #gelu #relu6 #_hard_swish #soft_plus #softsign #mish #cbrt
                    #celu #selu #elu #leaky_relu #hard_sigmoid
                }
            };
            ret.extend(res);
        } else {
            let trig = |func_name: &str, sleef_func: &str| {
                let func_name = Ident::new(func_name, proc_macro2::Span::call_site());
                let sleef_func = Ident::new(sleef_func, proc_macro2::Span::call_site());
                if res_type.is_f32() {
                    quote! {
                    fn #func_name(self) -> Self::Output {
                        paste::paste! {
                            #res_simd_ty::#res_simd_ty(sleef::f32x::#sleef_func(self.[<to_ #res_type>]().0))
                        }
                    }
                }
                } else {
                    quote! {
                        fn #func_name(self) -> Self::Output {
                            paste::paste! {
                                #res_simd_ty::#res_simd_ty(sleef::f64x::#sleef_func(self.[<to_ #res_type>]().0))
                            }
                        }
                    }
                }
            };
            let sin = trig("_sin", "sin_u35");
            let cos = trig("_cos", "cos_u35");
            let tan = trig("_tan", "tan_u35");
            let asin = trig("_asin", "asin_u35");
            let acos = trig("_acos", "acos_u35");
            let atan = trig("_atan", "atan_u35");
            let sinh = trig("_sinh", "sinh_u35");
            let cosh = trig("_cosh", "cosh_u35");
            let tanh = trig("_tanh", "tanh_u35");
            let asinh = trig("_asinh", "asinh_u10");
            let acosh = trig("_acosh", "acosh_u10");
            let atanh = trig("_atanh", "atanh_u10");
            let erf = trig("_erf", "erf_u10");
            let ln = trig("_ln", "log_u35");
            let exp = trig("_exp", "exp_u10");
            let exp2 = trig("_exp2", "exp2_u35");
            let log2 = trig("_log2", "log2_u35");
            let log10 = trig("_log10", "log10_u10");
            let sqrt = trig("_sqrt", "sqrt_u35");
            let cbrt = trig("_cbrt", "cbrt_u35");
            let res =
                quote! {
                impl FloatOutUnary for #lhs_simd {
                    type Output = #res_simd_ty::#res_simd_ty;
                    type Base = #res_type;
                    #sin #cos #tan #asin #acos #atan #sinh #cosh #tanh #asinh #acosh #atanh #erf #ln #exp #exp2 #log2 #log10 #sqrt
                    #cbrt
                    fn _recip(self) -> Self::Output {
                        paste::paste! {
                            #res_simd_ty::#res_simd_ty(self.[<to_ #res_type>]().recip())
                        }
                    }
                    fn _sigmoid(self) -> Self::Output {
                        paste::paste! {
                            #res_simd_ty::#res_simd_ty(
                                #res_simd_ty::#res_simd_ty::splat(#res_type::ONE).0 / (
                                    #res_simd_ty::#res_simd_ty::splat(#res_type::ONE).0 + (
                                        #res_simd_ty::#res_simd_ty(-self.[<to_ #res_type>]().0)
                                )._exp().0)
                            )
                        }
                    }
                    fn _relu(self) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]().0;
                            let mask = x.simd_gt(#res_simd_ty::#res_simd_ty::splat(#res_type::ZERO).0);
                            #res_simd_ty::#res_simd_ty(
                                mask.select(
                                    x, 
                                    #res_simd_ty::#res_simd_ty::splat(#res_type::ZERO).0
                            ))
                        }
                    }
                    fn _gelu(self) -> Self::Output {
                        paste::paste! {
                            let frac_1_sqrt_2 = #res_simd_ty::#res_simd_ty::splat(core::#res_type::consts::FRAC_1_SQRT_2);
                            let erf = (self.[<to_ #res_type>]() * frac_1_sqrt_2)._erf().0 + #res_simd_ty::#res_simd_ty::splat(#res_type::ONE).0;
                            #res_simd_ty::#res_simd_ty(#res_simd_ty::#res_simd_ty::splat(#res_type::HALF).0 * erf)
                        }
                    }
                    fn _relu6(self) -> Self::Output {
                        let relu = self._relu();
                        let six = #res_simd_ty::#res_simd_ty::splat(#res_type::SIX);
                        let min_mask = relu.simd_gt(six.0);
                        #res_simd_ty::#res_simd_ty(
                            min_mask.select(six.0, relu.0)
                        )
                    }
                    fn _softplus(self) -> Self::Output {
                        let one = #res_simd_ty::#res_simd_ty::splat(#res_type::ONE);
                        (one + self._exp())._ln()
                    }
                    fn _softsign(self) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]().0;
                            #res_simd_ty::#res_simd_ty(x / (#res_simd_ty::#res_simd_ty::splat(#res_type::ONE).0 + Sleef::abs(x)))
                        }
                    }
                    fn _mish(self) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            x * (#res_simd_ty::#res_simd_ty::splat(#res_type::ONE) + x._exp())._ln()._tanh()
                        }
                    }
                    fn _celu(self, alpha: Self::Base) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            let scale = #res_simd_ty::#res_simd_ty::splat(alpha);
                            let gt_mask = x.simd_gt(#res_simd_ty::#res_simd_ty::splat(#res_type::ZERO).0);
                            #res_simd_ty::#res_simd_ty(
                                gt_mask.select(
                                    x.0, 
                                    (scale * (x._exp() - #res_simd_ty::#res_simd_ty::splat(#res_type::ONE))).0
                                )
                            )
                        }
                    }
                    fn _selu(self, alpha: Self::Base, scale: Self::Base) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            let alpha = #res_simd_ty::#res_simd_ty::splat(alpha);
                            let scale = #res_simd_ty::#res_simd_ty::splat(scale);
                            let mask = x.simd_gt(#res_simd_ty::#res_simd_ty::splat(#res_type::ZERO).0);
                            #res_simd_ty::#res_simd_ty(
                                mask.select(scale.0 * x.0, scale.0 * alpha.0 * (x._exp().0 - #res_simd_ty::#res_simd_ty::splat(#res_type::ONE).0))
                            )
                        }
                    }
                    fn _elu(self, alpha: Self::Base) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            let alpha = #res_simd_ty::#res_simd_ty::splat(alpha);
                            let mask = x.simd_gt(#res_simd_ty::#res_simd_ty::splat(#res_type::ZERO).0);
                            #res_simd_ty::#res_simd_ty(
                                mask.select(x.0, (alpha * (x._exp() - #res_simd_ty::#res_simd_ty::splat(#res_type::ONE))).0)
                            )
                        }
                    }
                    fn _leaky_relu(self, alpha: Self::Base) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            let alpha = #res_simd_ty::#res_simd_ty::splat(alpha);
                            let mask = x.simd_gt(#res_simd_ty::#res_simd_ty::splat(#res_type::ZERO).0);
                            #res_simd_ty::#res_simd_ty(
                                mask.select(x.0, (alpha.0 * x.0))
                            )
                        }
                    }
                    fn _hard_swish(self) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            let six = #res_simd_ty::#res_simd_ty::splat(#res_type::SIX);
                            let three = #res_simd_ty::#res_simd_ty::splat(#res_type::THREE);
                            x * (x + three)._relu6() / six
                        }
                    }
                    fn _hard_sigmoid(self) -> Self::Output {
                        paste::paste! {
                            let x = self.[<to_ #res_type>]();
                            let point_two = #res_simd_ty::#res_simd_ty::splat(#res_type::POINT_TWO);
                            let half = #res_simd_ty::#res_simd_ty::splat(#res_type::HALF);
                            let one = #res_simd_ty::#res_simd_ty::splat(#res_type::ONE);
                            let zero = #res_simd_ty::#res_simd_ty::splat(#res_type::ZERO);
                            let add = point_two * x + half;
                            let mask = add.simd_gt(zero.0);
                            let select = mask.select(add.0, zero.0);
                            let min_mask = select.simd_lt(one.0);
                            let min_select = min_mask.select(select, one.0);
                            #res_simd_ty::#res_simd_ty(min_select)
                        }
                    }
                }
            };
            ret.extend(res);
        }
    }

    ret.into()
}

fn gen_func_arr(
    lhs_lanes: u8,
    res_type: Type,
    res_simd_ty: Ident,
    method: Ident
) -> proc_macro2::TokenStream {
    let unroll = (0..lhs_lanes as usize).map(|i| {
        quote! {
            arr[#i] = self_arr[#i].[<to_ #res_type>]().#method();
        }
    });
    let func =
        quote! {
        fn #method(self) -> Self::Output {
            paste::paste! {
                let mut arr = [#res_type::ZERO; #lhs_lanes as usize];
                let self_arr = self.0;
                #(#unroll)*
                #res_simd_ty::#res_simd_ty(arr.into())
            }
        }
    };
    func
}

fn unreachable_impl(
    lhs_simd: SimdType,
    res_ty: Type,
    res_simd_ty: Ident
) -> proc_macro2::TokenStream {
    quote! {
        impl FloatOutUnary for #lhs_simd {
            type Output = #res_simd_ty::#res_simd_ty;
            type Base = #res_ty;
            fn _exp(self) -> Self::Output {unreachable!()}
            fn _exp2(self) -> Self::Output {unreachable!()}
            fn _ln(self) -> Self::Output {unreachable!()}
            fn _celu(self, alpha: Self::Base) -> Self::Output {unreachable!()}
            fn _log2(self) -> Self::Output {unreachable!()}
            fn _log10(self) -> Self::Output {unreachable!()}
            fn _sqrt(self) -> Self::Output {unreachable!()}
            fn _sin(self) -> Self::Output {unreachable!()}
            fn _cos(self) -> Self::Output {unreachable!()}
            fn _tan(self) -> Self::Output {unreachable!()}
            fn _asin(self) -> Self::Output {unreachable!()}
            fn _acos(self) -> Self::Output {unreachable!()}
            fn _atan(self) -> Self::Output {unreachable!()}
            fn _sinh(self) -> Self::Output {unreachable!()}
            fn _cosh(self) -> Self::Output {unreachable!()}
            fn _tanh(self) -> Self::Output {unreachable!()}
            fn _asinh(self) -> Self::Output {unreachable!()}
            fn _acosh(self) -> Self::Output {unreachable!()}
            fn _atanh(self) -> Self::Output {unreachable!()}
            fn _recip(self) -> Self::Output {unreachable!()}
            fn _erf(self) -> Self::Output {unreachable!()}
            fn _sigmoid(self) -> Self::Output {unreachable!()}
            fn _elu(self, alpha: Self::Base) -> Self::Output {unreachable!()}
            fn _leaky_relu(self, alpha: Self::Base) -> Self::Output {unreachable!()}
            fn _relu(self) -> Self::Output {unreachable!()}
            fn _gelu(self) -> Self::Output {unreachable!()}
            fn _selu(self, alpha: Self::Base, scale: Self::Base) -> Self::Output {unreachable!()}
            fn _hard_sigmoid(self) -> Self::Output {unreachable!()}
            fn _relu6(self) -> Self::Output {unreachable!()}
            fn _hard_swish(self) -> Self::Output {unreachable!()}
            fn _softplus(self) -> Self::Output {unreachable!()}
            fn _softsign(self) -> Self::Output {unreachable!()}
            fn _mish(self) -> Self::Output {unreachable!()}
            fn _cbrt(self) -> Self::Output {unreachable!()}
        }
    }
}
