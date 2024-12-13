#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
use crate::arch_simd::sleef::arch::helper_avx2 as helper;
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "sse",
    not(target_feature = "avx2")
))]
use crate::arch_simd::sleef::arch::helper_sse as helper;

use helper::{
    vabs_vd_vd, vadd_vd_vd_vd, vadd_vi_vi_vi, vand_vi_vi_vi, vand_vi_vo_vi, vand_vm_vm_vm,
    vand_vm_vo64_vm, vand_vo_vo_vo, vandnot_vi_vi_vi, vandnot_vm_vo64_vm, vandnot_vo_vo_vo,
    vcast_vd_d, vcast_vd_vi, vcast_vi_i, vcast_vm_i_i, vcast_vo32_vo64, vcast_vo64_vo32,
    veq_vo_vd_vd, veq_vo_vi_vi, vfma_vd_vd_vd_vd, vfmanp_vd_vd_vd_vd, vfmapn_vd_vd_vd_vd,
    vgather_vd_p_vi, vge_vo_vd_vd, vgt_vo_vd_vd, vgt_vo_vi_vi, visinf_vo_vd, visnan_vo_vd,
    vispinf_vo_vd, vle_vo_vd_vd, vlt_vo_vd_vd, vmax_vd_vd_vd, vmin_vd_vd_vd, vmla_vd_vd_vd_vd,
    vmlapn_vd_vd_vd_vd, vmul_vd_vd_vd, vneg_vd_vd, vneg_vi_vi, vor_vm_vm_vm, vor_vm_vo64_vm,
    vor_vo_vo_vo, vreinterpret_vd_vm, vreinterpret_vm_vd, vrint_vd_vd, vrint_vi_vd, vsel_vd_vo_d_d,
    vsel_vd_vo_vd_vd, vsel_vi_vo_vi_vi, vsll_vi_vi_i, vsra_vi_vi_i, vsrl64_vm_vm_i,
    vsub64_vm_vm_vm, vsub_vd_vd_vd, vsub_vi_vi_vi, vtestallones_i_vo64, vtruncate_vd_vd,
    vtruncate_vi_vd, vxor_vm_vm_vm, vxor_vo_vo_vo,
};

use crate::{
    arch_simd::sleef::{
        common::{
            commonfuncs::{
                ddigetdd_vd2_ddi, ddigeti_vi_ddi, ddisetdd_ddi_ddi_vd2, ddisetddi_ddi_vd2_vi,
                digetd_vd_di, digeti_vi_di, rempisub, vilogb2k_vi_vd, vilogbk_vi_vd, visint_vo_vd,
                visnegzero_vo_vd, visodd_vo_vd, vldexp2_vd_vd_vi, vldexp3_vd_vd_vi,
                vmulsign_vd_vd_vd, vround2_vd_vd, vsignbit_vm_vd, vsignbit_vo_vd, vtruncate2_vd_vd,
                DDI,
            },
            dd::{
                ddadd2_vd2_vd2_vd, ddadd2_vd2_vd2_vd2, ddadd2_vd2_vd_vd, ddadd2_vd2_vd_vd2,
                ddadd_vd2_vd2_vd, ddadd_vd2_vd2_vd2, ddadd_vd2_vd_vd, ddadd_vd2_vd_vd2,
                dddiv_vd2_vd2_vd2, ddmul_vd2_vd2_vd, ddmul_vd2_vd2_vd2, ddmul_vd2_vd_vd,
                ddmul_vd_vd2_vd2, ddneg_vd2_vd2, ddnormalize_vd2_vd2, ddrec_vd2_vd, ddrec_vd2_vd2,
                ddscale_vd2_vd2_vd, ddsqrt_vd2_vd, ddsqrt_vd2_vd2, ddsqu_vd2_vd2, ddsqu_vd_vd2,
                ddsub_vd2_vd2_vd, ddsub_vd2_vd2_vd2, vcast_vd2_d_d, vcast_vd2_vd_vd,
                vd2getx_vd_vd2, vd2gety_vd_vd2, vd2setx_vd2_vd2_vd, vd2setxy_vd2_vd_vd,
                vd2sety_vd2_vd2_vd, vsel_vd2_vo_vd2_vd2, VDouble2,
            },
            estrin::{
                poly10d, poly12d, poly16d, poly21d, poly21d_, poly6d, poly7d, poly8d, poly9d,
            },
            misc::{
                L10_L, L10_U, L2L, L2U, LOG10_2, LOG1P_BOUND, LOG_DBL_MAX, M_1_PI, M_2_PI_H,
                M_2_PI_L, M_PI, PI_A, PI_A2, PI_B, PI_B2, PI_C, PI_D, R_LN2, SLEEF_DBL_MIN,
                SQRT_DBL_MAX, TRIGRANGEMAX, TRIGRANGEMAX2,
            },
        },
        table::SLEEF_REMPITABDP,
    },
    sleef_types::{VDouble, VInt, Vopmask},
};

#[inline(always)]
unsafe fn vsel_vi_vd_vd_vi_vi(d0: VDouble, d1: VDouble, x: VInt, y: VInt) -> VInt {
    vsel_vi_vo_vi_vi(vcast_vo32_vo64(vlt_vo_vd_vd(d0, d1)), x, y)
}

#[inline(always)]
unsafe fn vsel_vi_vd_vi(d: VDouble, x: VInt) -> VInt {
    vand_vi_vo_vi(vcast_vo32_vo64(vsignbit_vo_vd(d)), x)
}

#[inline(always)]
unsafe fn rempi(a: VDouble) -> DDI {
    let mut x: VDouble2;
    let mut y: VDouble2;

    // 计算指数
    let mut ex = vilogb2k_vi_vd(a);

    // AVX512 特定的处理
    #[cfg(target_feature = "avx512f")]
    {
        ex = vandnot_vi_vi_vi(vsra_vi_vi_i(ex, 31), ex);
        ex = vand_vi_vi_vi(ex, vcast_vi_i(1023));
    }

    // 调整指数
    ex = vsub_vi_vi_vi(ex, vcast_vi_i(55));

    // 处理大数
    let q = vand_vi_vo_vi(vgt_vo_vi_vi(ex, vcast_vi_i(700 - 55)), vcast_vi_i(-64));
    let a = vldexp3_vd_vd_vi(a, q);

    // 规范化指数
    ex = vandnot_vi_vi_vi(vsra_vi_vi_i::<31>(ex), ex);
    ex = vsll_vi_vi_i::<2>(ex);

    // 第一次乘法和归约
    x = ddmul_vd2_vd_vd(a, vgather_vd_p_vi(SLEEF_REMPITABDP.as_ptr(), ex));
    let mut di = rempisub(vd2getx_vd_vd2(x));
    let mut q = digeti_vi_di(di);
    x = vd2setx_vd2_vd2_vd(x, digetd_vd_di(di));
    x = ddnormalize_vd2_vd2(x);

    // 第二次乘法和归约
    y = ddmul_vd2_vd_vd(a, vgather_vd_p_vi(SLEEF_REMPITABDP.as_ptr().add(1), ex));
    x = ddadd2_vd2_vd2_vd2(x, y);
    di = rempisub(vd2getx_vd_vd2(x));
    q = vadd_vi_vi_vi(q, digeti_vi_di(di));
    x = vd2setx_vd2_vd2_vd(x, digetd_vd_di(di));
    x = ddnormalize_vd2_vd2(x);

    // 第三次乘法和归约
    y = vcast_vd2_vd_vd(
        vgather_vd_p_vi(SLEEF_REMPITABDP.as_ptr().add(2), ex),
        vgather_vd_p_vi(SLEEF_REMPITABDP.as_ptr().add(3), ex),
    );
    y = ddmul_vd2_vd2_vd(y, a);
    x = ddadd2_vd2_vd2_vd2(x, y);
    x = ddnormalize_vd2_vd2(x);

    // 最终乘以 2π
    x = ddmul_vd2_vd2_vd2(
        x,
        vcast_vd2_d_d(3.141592653589793116 * 2.0, 1.2246467991473532072e-16 * 2.0),
    );

    // 处理小数
    let o = vlt_vo_vd_vd(vabs_vd_vd(a), vcast_vd_d(0.7));
    x = vd2setx_vd2_vd2_vd(x, vsel_vd_vo_vd_vd(o, a, vd2getx_vd_vd2(x)));
    x = vd2sety_vd2_vd2_vd(
        x,
        vreinterpret_vd_vm(vandnot_vm_vo64_vm(o, vreinterpret_vm_vd(vd2gety_vd_vd2(x)))),
    );

    // 返回结果
    ddisetddi_ddi_vd2_vi(x, q)
}

#[inline(always)]
pub(crate) unsafe fn xsin_u1(d: VDouble) -> VDouble {
    let mut u: VDouble;
    let mut s: VDouble2;
    let t: VDouble2;
    let mut x: VDouble2;
    let mut ql: VInt;

    // First range check
    let mut g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX2));
    let dql = vrint_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(M_1_PI)));
    ql = vrint_vi_vd(dql);
    u = vmla_vd_vd_vd_vd(dql, vcast_vd_d(-PI_A2), d);
    x = ddadd_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B2)));

    // Medium range handling
    if vtestallones_i_vo64(g) == 0 {
        let mut dqh = vtruncate_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(M_1_PI / (1 << 24) as f64)));
        dqh = vmul_vd_vd_vd(dqh, vcast_vd_d((1 << 24) as f64));
        let dql = vrint_vd_vd(vmlapn_vd_vd_vd_vd(d, vcast_vd_d(M_1_PI), dqh));

        u = vmla_vd_vd_vd_vd(dqh, vcast_vd_d(-PI_A), d);
        s = ddadd_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_A)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_B)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_C)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_C)));
        s = ddadd_vd2_vd2_vd(s, vmul_vd_vd_vd(vadd_vd_vd_vd(dqh, dql), vcast_vd_d(-PI_D)));

        ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, vrint_vi_vd(dql));
        x = vsel_vd2_vo_vd2_vd2(g, x, s);
        g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX));

        // Large range handling
        if vtestallones_i_vo64(g) == 0 {
            let ddi = rempi(d);
            let mut ql2 = vand_vi_vi_vi(ddigeti_vi_ddi(ddi), vcast_vi_i(3));
            ql2 = vadd_vi_vi_vi(
                vadd_vi_vi_vi(ql2, ql2),
                vsel_vi_vo_vi_vi(
                    vcast_vo32_vo64(vgt_vo_vd_vd(
                        vd2getx_vd_vd2(ddigetdd_vd2_ddi(ddi)),
                        vcast_vd_d(0.0),
                    )),
                    vcast_vi_i(2),
                    vcast_vi_i(1),
                ),
            );
            ql2 = vsra_vi_vi_i::<2>(ql2);

            let o = veq_vo_vi_vi(
                vand_vi_vi_vi(ddigeti_vi_ddi(ddi), vcast_vi_i(1)),
                vcast_vi_i(1),
            );

            let mut t = vcast_vd2_vd_vd(
                vmulsign_vd_vd_vd(
                    vcast_vd_d(-3.141592653589793116 * 0.5),
                    vd2getx_vd_vd2(ddigetdd_vd2_ddi(ddi)),
                ),
                vmulsign_vd_vd_vd(
                    vcast_vd_d(-1.2246467991473532072e-16 * 0.5),
                    vd2getx_vd_vd2(ddigetdd_vd2_ddi(ddi)),
                ),
            );

            t = ddadd2_vd2_vd2_vd2(ddigetdd_vd2_ddi(ddi), t);
            let ddi = ddisetdd_ddi_ddi_vd2(
                ddi,
                vsel_vd2_vo_vd2_vd2(vcast_vo64_vo32(o), t, ddigetdd_vd2_ddi(ddi)),
            );
            s = ddnormalize_vd2_vd2(ddigetdd_vd2_ddi(ddi));
            ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, ql2);
            x = vsel_vd2_vo_vd2_vd2(g, x, s);
            x = vd2setx_vd2_vd2_vd(
                x,
                vreinterpret_vd_vm(vor_vm_vo64_vm(
                    vor_vo_vo_vo(visinf_vo_vd(d), visnan_vo_vd(d)),
                    vreinterpret_vm_vd(vd2getx_vd_vd2(x)),
                )),
            );
        }
    }

    // Taylor series approximation
    t = x;
    s = ddsqu_vd2_vd2(x);

    let s2 = vmul_vd_vd_vd(vd2getx_vd_vd2(s), vd2getx_vd_vd2(s));
    let s4 = vmul_vd_vd_vd(s2, s2);

    // Polynomial evaluation
    u = poly6d(
        vd2getx_vd_vd2(s),
        s2,
        s4,
        2.72052416138529567917983e-15,
        -7.6429259411395447190023e-13,
        1.60589370117277896211623e-10,
        -2.5052106814843123359368e-08,
        2.75573192104428224777379e-06,
        -0.000198412698412046454654947,
    );

    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(0.00833333333333318056201922),
    );

    // Final computations
    x = ddadd_vd2_vd_vd2(
        vcast_vd_d(1.0),
        ddmul_vd2_vd2_vd2(
            ddadd_vd2_vd_vd(
                vcast_vd_d(-0.166666666666666657414808),
                vmul_vd_vd_vd(u, vd2getx_vd_vd2(s)),
            ),
            s,
        ),
    );
    u = ddmul_vd_vd2_vd2(t, x);

    // Sign handling
    u = vreinterpret_vd_vm(vxor_vm_vm_vm(
        vand_vm_vo64_vm(
            vcast_vo64_vo32(veq_vo_vi_vi(
                vand_vi_vi_vi(ql, vcast_vi_i(1)),
                vcast_vi_i(1),
            )),
            vreinterpret_vm_vd(vcast_vd_d(-0.0)),
        ),
        vreinterpret_vm_vd(u),
    ));

    // Handle special case for zero
    vsel_vd_vo_vd_vd(veq_vo_vd_vd(d, vcast_vd_d(0.0)), d, u)
}

#[inline(always)]
pub(crate) unsafe fn xcos_u1(d: VDouble) -> VDouble {
    let mut u: VDouble;
    let mut s: VDouble2;
    let t: VDouble2;
    let mut x: VDouble2;
    let mut ql: VInt;

    // First range check
    let mut g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX2));
    let mut dql = vrint_vd_vd(vmla_vd_vd_vd_vd(d, vcast_vd_d(M_1_PI), vcast_vd_d(-0.5)));
    dql = vmla_vd_vd_vd_vd(vcast_vd_d(2.0), dql, vcast_vd_d(1.0));
    ql = vrint_vi_vd(dql);
    x = ddadd2_vd2_vd_vd(d, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_A2 * 0.5)));
    x = ddadd_vd2_vd2_vd(x, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B2 * 0.5)));

    // Medium range handling
    if vtestallones_i_vo64(g) == 0 {
        let mut dqh = vtruncate_vd_vd(vmla_vd_vd_vd_vd(
            d,
            vcast_vd_d(M_1_PI / (1 << 23) as f64),
            vcast_vd_d(-M_1_PI / (1 << 24) as f64),
        ));
        let mut ql2 = vrint_vi_vd(vadd_vd_vd_vd(
            vmul_vd_vd_vd(d, vcast_vd_d(M_1_PI)),
            vmla_vd_vd_vd_vd(dqh, vcast_vd_d(-(1 << 23) as f64), vcast_vd_d(-0.5)),
        ));
        dqh = vmul_vd_vd_vd(dqh, vcast_vd_d((1 << 24) as f64));
        ql2 = vadd_vi_vi_vi(vadd_vi_vi_vi(ql2, ql2), vcast_vi_i(1));
        let dql = vcast_vd_vi(ql2);

        u = vmla_vd_vd_vd_vd(dqh, vcast_vd_d(-PI_A * 0.5), d);
        s = ddadd2_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_A * 0.5)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_B * 0.5)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B * 0.5)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_C * 0.5)));
        s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_C * 0.5)));
        s = ddadd_vd2_vd2_vd(
            s,
            vmul_vd_vd_vd(vadd_vd_vd_vd(dqh, dql), vcast_vd_d(-PI_D * 0.5)),
        );

        ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, ql2);
        x = vsel_vd2_vo_vd2_vd2(g, x, s);
        g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX));

        // Large range handling
        if vtestallones_i_vo64(g) == 0 {
            let ddi = rempi(d);
            let mut ql2 = vand_vi_vi_vi(ddigeti_vi_ddi(ddi), vcast_vi_i(3));
            ql2 = vadd_vi_vi_vi(
                vadd_vi_vi_vi(ql2, ql2),
                vsel_vi_vo_vi_vi(
                    vcast_vo32_vo64(vgt_vo_vd_vd(
                        vd2getx_vd_vd2(ddigetdd_vd2_ddi(ddi)),
                        vcast_vd_d(0.0),
                    )),
                    vcast_vi_i(8),
                    vcast_vi_i(7),
                ),
            );
            ql2 = vsra_vi_vi_i::<1>(ql2);

            let o = veq_vo_vi_vi(
                vand_vi_vi_vi(ddigeti_vi_ddi(ddi), vcast_vi_i(1)),
                vcast_vi_i(0),
            );

            let y = vsel_vd_vo_vd_vd(
                vgt_vo_vd_vd(vd2getx_vd_vd2(ddigetdd_vd2_ddi(ddi)), vcast_vd_d(0.0)),
                vcast_vd_d(0.0),
                vcast_vd_d(-1.0),
            );

            let mut t = vcast_vd2_vd_vd(
                vmulsign_vd_vd_vd(vcast_vd_d(-3.141592653589793116 * 0.5), y),
                vmulsign_vd_vd_vd(vcast_vd_d(-1.2246467991473532072e-16 * 0.5), y),
            );

            t = ddadd2_vd2_vd2_vd2(ddigetdd_vd2_ddi(ddi), t);
            let ddi = ddisetdd_ddi_ddi_vd2(
                ddi,
                vsel_vd2_vo_vd2_vd2(vcast_vo64_vo32(o), t, ddigetdd_vd2_ddi(ddi)),
            );
            s = ddnormalize_vd2_vd2(ddigetdd_vd2_ddi(ddi));
            ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, ql2);
            x = vsel_vd2_vo_vd2_vd2(g, x, s);
            x = vd2setx_vd2_vd2_vd(
                x,
                vreinterpret_vd_vm(vor_vm_vo64_vm(
                    vor_vo_vo_vo(visinf_vo_vd(d), visnan_vo_vd(d)),
                    vreinterpret_vm_vd(vd2getx_vd_vd2(x)),
                )),
            );
        }
    }

    // Taylor series approximation
    t = x;
    s = ddsqu_vd2_vd2(x);

    let s2 = vmul_vd_vd_vd(vd2getx_vd_vd2(s), vd2getx_vd_vd2(s));
    let s4 = vmul_vd_vd_vd(s2, s2);

    // Polynomial evaluation
    u = poly6d(
        vd2getx_vd_vd2(s),
        s2,
        s4,
        2.72052416138529567917983e-15,
        -7.6429259411395447190023e-13,
        1.60589370117277896211623e-10,
        -2.5052106814843123359368e-08,
        2.75573192104428224777379e-06,
        -0.000198412698412046454654947,
    );

    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(0.00833333333333318056201922),
    );

    // Final computations
    x = ddadd_vd2_vd_vd2(
        vcast_vd_d(1.0),
        ddmul_vd2_vd2_vd2(
            ddadd_vd2_vd_vd(
                vcast_vd_d(-0.166666666666666657414808),
                vmul_vd_vd_vd(u, vd2getx_vd_vd2(s)),
            ),
            s,
        ),
    );
    u = ddmul_vd_vd2_vd2(t, x);

    // Sign handling
    u = vreinterpret_vd_vm(vxor_vm_vm_vm(
        vand_vm_vo64_vm(
            vcast_vo64_vo32(veq_vo_vi_vi(
                vand_vi_vi_vi(ql, vcast_vi_i(2)),
                vcast_vi_i(0),
            )),
            vreinterpret_vm_vd(vcast_vd_d(-0.0)),
        ),
        vreinterpret_vm_vd(u),
    ));

    u
}

#[inline(always)]
pub(crate) unsafe fn xsincos_u1(d: VDouble) -> VDouble2 {
    let mut o: Vopmask;
    let mut u: VDouble;
    let mut rx: VDouble;
    let ry: VDouble;
    let mut r: VDouble2;
    let mut s: VDouble2;
    let t: VDouble2;
    let mut x: VDouble2;
    let mut ql: VInt;

    // Initial range reduction
    let dql = vrint_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(2.0 * M_1_PI)));
    ql = vrint_vi_vd(dql);
    u = vmla_vd_vd_vd_vd(dql, vcast_vd_d(-PI_A2 * 0.5), d);
    s = ddadd_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B2 * 0.5)));
    let mut g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX2));

    // Medium range handling
    if vtestallones_i_vo64(g) == 0 {
        let mut dqh = vtruncate_vd_vd(vmul_vd_vd_vd(
            d,
            vcast_vd_d(2.0 * M_1_PI / (1 << 24) as f64),
        ));
        dqh = vmul_vd_vd_vd(dqh, vcast_vd_d((1 << 24) as f64));
        let dql = vrint_vd_vd(vsub_vd_vd_vd(
            vmul_vd_vd_vd(d, vcast_vd_d(2.0 * M_1_PI)),
            dqh,
        ));

        u = vmla_vd_vd_vd_vd(dqh, vcast_vd_d(-PI_A * 0.5), d);
        x = ddadd_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_A * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_B * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_C * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_C * 0.5)));
        x = ddadd_vd2_vd2_vd(
            x,
            vmul_vd_vd_vd(vadd_vd_vd_vd(dqh, dql), vcast_vd_d(-PI_D * 0.5)),
        );

        ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, vrint_vi_vd(dql));
        s = vsel_vd2_vo_vd2_vd2(g, s, x);
        g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX));

        // Large range handling
        if vtestallones_i_vo64(g) == 0 {
            let ddi = rempi(d);
            x = ddigetdd_vd2_ddi(ddi);
            o = vor_vo_vo_vo(visinf_vo_vd(d), visnan_vo_vd(d));
            x = vd2setx_vd2_vd2_vd(
                x,
                vreinterpret_vd_vm(vor_vm_vo64_vm(o, vreinterpret_vm_vd(vd2getx_vd_vd2(x)))),
            );
            x = vd2sety_vd2_vd2_vd(
                x,
                vreinterpret_vd_vm(vor_vm_vo64_vm(o, vreinterpret_vm_vd(vd2gety_vd_vd2(x)))),
            );

            ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, ddigeti_vi_ddi(ddi));
            s = vsel_vd2_vo_vd2_vd2(g, s, x);
        }
    }

    // Calculate sine
    t = s;
    s = vd2setx_vd2_vd2_vd(s, ddsqu_vd_vd2(s));

    // Sine polynomial
    u = vcast_vd_d(1.58938307283228937328511e-10);
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(-2.50506943502539773349318e-08),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(2.75573131776846360512547e-06),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(-0.000198412698278911770864914),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(0.0083333333333191845961746),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(-0.166666666666666130709393),
    );

    u = vmul_vd_vd_vd(u, vmul_vd_vd_vd(vd2getx_vd_vd2(s), vd2getx_vd_vd2(t)));

    x = ddadd_vd2_vd2_vd(t, u);
    rx = vadd_vd_vd_vd(vd2getx_vd_vd2(x), vd2gety_vd_vd2(x));

    rx = vsel_vd_vo_vd_vd(visnegzero_vo_vd(d), vcast_vd_d(-0.0), rx);

    // Cosine polynomial
    u = vcast_vd_d(-1.13615350239097429531523e-11);
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(2.08757471207040055479366e-09),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(-2.75573144028847567498567e-07),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(2.48015872890001867311915e-05),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(-0.00138888888888714019282329),
    );
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(s),
        vcast_vd_d(0.0416666666666665519592062),
    );
    u = vmla_vd_vd_vd_vd(u, vd2getx_vd_vd2(s), vcast_vd_d(-0.5));

    x = ddadd_vd2_vd_vd2(vcast_vd_d(1.0), ddmul_vd2_vd_vd(vd2getx_vd_vd2(s), u));
    ry = vadd_vd_vd_vd(vd2getx_vd_vd2(x), vd2gety_vd_vd2(x));

    // Quadrant handling
    o = vcast_vo64_vo32(veq_vo_vi_vi(
        vand_vi_vi_vi(ql, vcast_vi_i(1)),
        vcast_vi_i(0),
    ));
    r = vd2setxy_vd2_vd_vd(vsel_vd_vo_vd_vd(o, rx, ry), vsel_vd_vo_vd_vd(o, ry, rx));

    // Sign handling for sine
    o = vcast_vo64_vo32(veq_vo_vi_vi(
        vand_vi_vi_vi(ql, vcast_vi_i(2)),
        vcast_vi_i(2),
    ));
    r = vd2setx_vd2_vd2_vd(
        r,
        vreinterpret_vd_vm(vxor_vm_vm_vm(
            vand_vm_vo64_vm(o, vreinterpret_vm_vd(vcast_vd_d(-0.0))),
            vreinterpret_vm_vd(vd2getx_vd_vd2(r)),
        )),
    );

    // Sign handling for cosine
    o = vcast_vo64_vo32(veq_vo_vi_vi(
        vand_vi_vi_vi(vadd_vi_vi_vi(ql, vcast_vi_i(1)), vcast_vi_i(2)),
        vcast_vi_i(2),
    ));
    r = vd2sety_vd2_vd2_vd(
        r,
        vreinterpret_vd_vm(vxor_vm_vm_vm(
            vand_vm_vo64_vm(o, vreinterpret_vm_vd(vcast_vd_d(-0.0))),
            vreinterpret_vm_vd(vd2gety_vd_vd2(r)),
        )),
    );

    r
}

#[inline(always)]
pub(crate) unsafe fn xtan_u1(d: VDouble) -> VDouble {
    let mut u: VDouble;
    let mut s: VDouble2;
    let t: VDouble2;
    let mut x: VDouble2;
    let y: VDouble2;
    let mut o: Vopmask;
    let mut ql: VInt;

    // Initial range reduction
    let dql = vrint_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(2.0 * M_1_PI)));
    ql = vrint_vi_vd(dql);
    u = vmla_vd_vd_vd_vd(dql, vcast_vd_d(-PI_A2 * 0.5), d);
    s = ddadd_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B2 * 0.5)));
    let mut g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX2));

    // Medium range handling
    if vtestallones_i_vo64(g) == 0 {
        let mut dqh = vtruncate_vd_vd(vmul_vd_vd_vd(
            d,
            vcast_vd_d(2.0 * M_1_PI / (1 << 24) as f64),
        ));
        dqh = vmul_vd_vd_vd(dqh, vcast_vd_d((1 << 24) as f64));
        x = ddadd2_vd2_vd2_vd(
            ddmul_vd2_vd2_vd(vcast_vd2_d_d(M_2_PI_H, M_2_PI_L), d),
            vsub_vd_vd_vd(
                vsel_vd_vo_vd_vd(
                    vlt_vo_vd_vd(d, vcast_vd_d(0.0)),
                    vcast_vd_d(-0.5),
                    vcast_vd_d(0.5),
                ),
                dqh,
            ),
        );
        let dql = vtruncate_vd_vd(vadd_vd_vd_vd(vd2getx_vd_vd2(x), vd2gety_vd_vd2(x)));

        u = vmla_vd_vd_vd_vd(dqh, vcast_vd_d(-PI_A * 0.5), d);
        x = ddadd_vd2_vd_vd(u, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_A * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_B * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_B * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dqh, vcast_vd_d(-PI_C * 0.5)));
        x = ddadd2_vd2_vd2_vd(x, vmul_vd_vd_vd(dql, vcast_vd_d(-PI_C * 0.5)));
        x = ddadd_vd2_vd2_vd(
            x,
            vmul_vd_vd_vd(vadd_vd_vd_vd(dqh, dql), vcast_vd_d(-PI_D * 0.5)),
        );

        ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, vrint_vi_vd(dql));
        s = vsel_vd2_vo_vd2_vd2(g, s, x);
        g = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(TRIGRANGEMAX));

        // Large range handling
        if vtestallones_i_vo64(g) == 0 {
            let ddi = rempi(d);
            x = ddigetdd_vd2_ddi(ddi);
            o = vor_vo_vo_vo(visinf_vo_vd(d), visnan_vo_vd(d));
            x = vd2setx_vd2_vd2_vd(
                x,
                vreinterpret_vd_vm(vor_vm_vo64_vm(o, vreinterpret_vm_vd(vd2getx_vd_vd2(x)))),
            );
            x = vd2sety_vd2_vd2_vd(
                x,
                vreinterpret_vd_vm(vor_vm_vo64_vm(o, vreinterpret_vm_vd(vd2gety_vd_vd2(x)))),
            );

            ql = vsel_vi_vo_vi_vi(vcast_vo32_vo64(g), ql, ddigeti_vi_ddi(ddi));
            s = vsel_vd2_vo_vd2_vd2(g, s, x);
        }
    }

    // Polynomial approximation
    t = ddscale_vd2_vd2_vd(s, vcast_vd_d(0.5));
    s = ddsqu_vd2_vd2(t);

    let s2 = vmul_vd_vd_vd(vd2getx_vd_vd2(s), vd2getx_vd_vd2(s));
    let s4 = vmul_vd_vd_vd(s2, s2);

    u = poly8d(
        vd2getx_vd_vd2(s),
        s2,
        s4,
        0.3245098826639276316e-3,
        0.5619219738114323735e-3,
        0.1460781502402784494e-2,
        0.3591611540792499519e-2,
        0.8863268409563113126e-2,
        0.2186948728185535498e-1,
        0.5396825399517272970e-1,
        0.1333333333330500581e+0,
    );

    u = vmla_vd_vd_vd_vd(u, vd2getx_vd_vd2(s), vcast_vd_d(0.3333333333333343695e+0));
    x = ddadd_vd2_vd2_vd2(t, ddmul_vd2_vd2_vd(ddmul_vd2_vd2_vd2(s, t), u));

    // Final computations
    y = ddadd_vd2_vd_vd2(vcast_vd_d(-1.0), ddsqu_vd2_vd2(x));
    x = ddscale_vd2_vd2_vd(x, vcast_vd_d(-2.0));

    o = vcast_vo64_vo32(veq_vo_vi_vi(
        vand_vi_vi_vi(ql, vcast_vi_i(1)),
        vcast_vi_i(1),
    ));

    x = dddiv_vd2_vd2_vd2(
        vsel_vd2_vo_vd2_vd2(o, ddneg_vd2_vd2(y), x),
        vsel_vd2_vo_vd2_vd2(o, x, y),
    );

    u = vadd_vd_vd_vd(vd2getx_vd_vd2(x), vd2gety_vd_vd2(x));

    // Handle special case for zero
    vsel_vd_vo_vd_vd(veq_vo_vd_vd(d, vcast_vd_d(0.0)), d, u)
}

#[inline(always)]
pub(crate) unsafe fn atan2k_u1(y: VDouble2, x: VDouble2) -> VDouble2 {
    let mut u: VDouble;
    let mut s: VDouble2;
    let mut t: VDouble2;
    let mut q: VInt;
    let mut p: Vopmask;

    // Handle x < 0 case
    q = vsel_vi_vd_vi(vd2getx_vd_vd2(x), vcast_vi_i(-2));
    p = vlt_vo_vd_vd(vd2getx_vd_vd2(x), vcast_vd_d(0.0));
    let b = vand_vm_vo64_vm(p, vreinterpret_vm_vd(vcast_vd_d(-0.0)));
    let x = vd2setx_vd2_vd2_vd(
        x,
        vreinterpret_vd_vm(vxor_vm_vm_vm(b, vreinterpret_vm_vd(vd2getx_vd_vd2(x)))),
    );
    let x = vd2sety_vd2_vd2_vd(
        x,
        vreinterpret_vd_vm(vxor_vm_vm_vm(b, vreinterpret_vm_vd(vd2gety_vd_vd2(x)))),
    );

    // Determine quadrant
    q = vsel_vi_vd_vd_vi_vi(
        vd2getx_vd_vd2(x),
        vd2getx_vd_vd2(y),
        vadd_vi_vi_vi(q, vcast_vi_i(1)),
        q,
    );
    p = vlt_vo_vd_vd(vd2getx_vd_vd2(x), vd2getx_vd_vd2(y));
    s = vsel_vd2_vo_vd2_vd2(p, ddneg_vd2_vd2(x), y);
    t = vsel_vd2_vo_vd2_vd2(p, y, x);

    // Calculate ratio and square
    s = dddiv_vd2_vd2_vd2(s, t);
    t = ddsqu_vd2_vd2(s);
    t = ddnormalize_vd2_vd2(t);

    // Polynomial evaluation
    let t2 = vmul_vd_vd_vd(vd2getx_vd_vd2(t), vd2getx_vd_vd2(t));
    let t4 = vmul_vd_vd_vd(t2, t2);
    let t8 = vmul_vd_vd_vd(t4, t4);

    u = poly16d(
        vd2getx_vd_vd2(t),
        t2,
        t4,
        t8,
        1.06298484191448746607415e-05,
        -0.000125620649967286867384336,
        0.00070557664296393412389774,
        -0.00251865614498713360352999,
        0.00646262899036991172313504,
        -0.0128281333663399031014274,
        0.0208024799924145797902497,
        -0.0289002344784740315686289,
        0.0359785005035104590853656,
        -0.041848579703592507506027,
        0.0470843011653283988193763,
        -0.0524914210588448421068719,
        0.0587946590969581003860434,
        -0.0666620884778795497194182,
        0.0769225330296203768654095,
        -0.0909090442773387574781907,
    );

    // Final polynomial terms
    u = vmla_vd_vd_vd_vd(u, vd2getx_vd_vd2(t), vcast_vd_d(0.111111108376896236538123));
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(t),
        vcast_vd_d(-0.142857142756268568062339),
    );
    u = vmla_vd_vd_vd_vd(u, vd2getx_vd_vd2(t), vcast_vd_d(0.199999999997977351284817));
    u = vmla_vd_vd_vd_vd(
        u,
        vd2getx_vd_vd2(t),
        vcast_vd_d(-0.333333333333317605173818),
    );

    // Combine results
    t = ddadd_vd2_vd2_vd2(s, ddmul_vd2_vd2_vd(ddmul_vd2_vd2_vd2(s, t), u));

    // Add pi/2 * q
    t = ddadd_vd2_vd2_vd2(
        ddmul_vd2_vd2_vd(
            vcast_vd2_d_d(1.570796326794896557998982, 6.12323399573676603586882e-17),
            vcast_vd_vi(q),
        ),
        t,
    );

    t
}

#[inline(always)]
pub(crate) unsafe fn visinf2_vd_vd_vd(d: VDouble, m: VDouble) -> VDouble {
    // Returns m with the sign of d if d is infinite, 0 otherwise
    vreinterpret_vd_vm(vand_vm_vo64_vm(
        visinf_vo_vd(d), // Mask for infinite values
        vor_vm_vm_vm(
            vand_vm_vm_vm(
                vreinterpret_vm_vd(d),
                vreinterpret_vm_vd(vcast_vd_d(-0.0)), // Extract sign bit
            ),
            vreinterpret_vm_vd(m), // Original value
        ),
    ))
}

#[inline(always)]
pub(crate) unsafe fn xatan2_u1(y: VDouble, x: VDouble) -> VDouble {
    // Handle subnormal numbers by scaling up
    let o = vlt_vo_vd_vd(
        vabs_vd_vd(x),
        vcast_vd_d(5.5626846462680083984e-309), // nexttoward((1.0 / DBL_MAX), 1)
    );
    let x = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(x, vcast_vd_d((1u64 << 53) as f64)), x);
    let y = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(y, vcast_vd_d((1u64 << 53) as f64)), y);

    // Calculate atan2 using double-double arithmetic
    let d = atan2k_u1(
        vcast_vd2_vd_vd(vabs_vd_vd(y), vcast_vd_d(0.0)),
        vcast_vd2_vd_vd(x, vcast_vd_d(0.0)),
    );
    let mut r = vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d));

    // Apply sign of x
    r = vmulsign_vd_vd_vd(r, x);

    // Handle special cases for x
    r = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(visinf_vo_vd(x), veq_vo_vd_vd(x, vcast_vd_d(0.0))),
        vsub_vd_vd_vd(
            vcast_vd_d(M_PI / 2.0),
            visinf2_vd_vd_vd(x, vmulsign_vd_vd_vd(vcast_vd_d(M_PI / 2.0), x)),
        ),
        r,
    );

    // Handle infinite y
    r = vsel_vd_vo_vd_vd(
        visinf_vo_vd(y),
        vsub_vd_vd_vd(
            vcast_vd_d(M_PI / 2.0),
            visinf2_vd_vd_vd(x, vmulsign_vd_vd_vd(vcast_vd_d(M_PI / 4.0), x)),
        ),
        r,
    );

    // Handle zero y
    r = vsel_vd_vo_vd_vd(
        veq_vo_vd_vd(y, vcast_vd_d(0.0)),
        vreinterpret_vd_vm(vand_vm_vo64_vm(
            vsignbit_vo_vd(x),
            vreinterpret_vm_vd(vcast_vd_d(M_PI)),
        )),
        r,
    );

    // Handle NaN inputs and apply sign of y
    r = vreinterpret_vd_vm(vor_vm_vo64_vm(
        vor_vo_vo_vo(visnan_vo_vd(x), visnan_vo_vd(y)),
        vreinterpret_vm_vd(vmulsign_vd_vd_vd(r, y)),
    ));

    r
}

#[inline(always)]
pub(crate) unsafe fn xasin_u1(d: VDouble) -> VDouble {
    // Check if |d| < 0.5
    let o = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(0.5));

    // Calculate x² based on input range
    let x2 = vsel_vd_vo_vd_vd(
        o,
        vmul_vd_vd_vd(d, d), // d² for small inputs
        vmul_vd_vd_vd(
            // 0.5(1-|d|) for larger inputs
            vsub_vd_vd_vd(vcast_vd_d(1.0), vabs_vd_vd(d)),
            vcast_vd_d(0.5),
        ),
    );

    let mut u: VDouble;

    // Calculate square root for larger inputs
    let mut x = vsel_vd2_vo_vd2_vd2(
        o,
        vcast_vd2_vd_vd(vabs_vd_vd(d), vcast_vd_d(0.0)),
        ddsqrt_vd2_vd(x2),
    );

    // Handle edge case |d| = 1
    x = vsel_vd2_vo_vd2_vd2(
        veq_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(1.0)),
        vcast_vd2_d_d(0.0, 0.0),
        x,
    );

    // Calculate powers of x
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    let x16 = vmul_vd_vd_vd(x8, x8);

    // Polynomial approximation
    u = poly12d(
        x2,
        x4,
        x8,
        x16,
        0.3161587650653934628e-1,
        -0.1581918243329996643e-1,
        0.1929045477267910674e-1,
        0.6606077476277170610e-2,
        0.1215360525577377331e-1,
        0.1388715184501609218e-1,
        0.1735956991223614604e-1,
        0.2237176181932048341e-1,
        0.3038195928038132237e-1,
        0.4464285681377102438e-1,
        0.7500000000378581611e-1,
        0.1666666666666497543e+0,
    );

    // Multiply by x²x
    u = vmul_vd_vd_vd(u, vmul_vd_vd_vd(x2, vd2getx_vd_vd2(x)));

    // Calculate π/4 - x - u using double-double arithmetic
    let y = ddsub_vd2_vd2_vd(
        ddsub_vd2_vd2_vd2(
            vcast_vd2_d_d(3.141592653589793116 / 4.0, 1.2246467991473532072e-16 / 4.0),
            x,
        ),
        u,
    );

    // Select final result based on input range
    let r = vsel_vd_vo_vd_vd(
        o,
        vadd_vd_vd_vd(u, vd2getx_vd_vd2(x)),
        vmul_vd_vd_vd(
            vadd_vd_vd_vd(vd2getx_vd_vd2(y), vd2gety_vd_vd2(y)),
            vcast_vd_d(2.0),
        ),
    );

    // Apply sign of input
    vmulsign_vd_vd_vd(r, d)
}

#[inline(always)]
pub(crate) unsafe fn xacos_u1(d: VDouble) -> VDouble {
    // Check if |d| < 0.5
    let o = vlt_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(0.5));

    // Calculate x² based on input range
    let x2 = vsel_vd_vo_vd_vd(
        o,
        vmul_vd_vd_vd(d, d), // d² for small inputs
        vmul_vd_vd_vd(
            // 0.5(1-|d|) for larger inputs
            vsub_vd_vd_vd(vcast_vd_d(1.0), vabs_vd_vd(d)),
            vcast_vd_d(0.5),
        ),
    );
    let mut u: VDouble;

    // Calculate square root for larger inputs
    let mut x = vsel_vd2_vo_vd2_vd2(
        o,
        vcast_vd2_vd_vd(vabs_vd_vd(d), vcast_vd_d(0.0)),
        ddsqrt_vd2_vd(x2),
    );

    // Handle edge case |d| = 1
    x = vsel_vd2_vo_vd2_vd2(
        veq_vo_vd_vd(vabs_vd_vd(d), vcast_vd_d(1.0)),
        vcast_vd2_d_d(0.0, 0.0),
        x,
    );

    // Calculate powers of x
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    let x16 = vmul_vd_vd_vd(x8, x8);

    // Polynomial approximation
    u = poly12d(
        x2,
        x4,
        x8,
        x16,
        0.3161587650653934628e-1,
        -0.1581918243329996643e-1,
        0.1929045477267910674e-1,
        0.6606077476277170610e-2,
        0.1215360525577377331e-1,
        0.1388715184501609218e-1,
        0.1735956991223614604e-1,
        0.2237176181932048341e-1,
        0.3038195928038132237e-1,
        0.4464285681377102438e-1,
        0.7500000000378581611e-1,
        0.1666666666666497543e+0,
    );

    // Multiply by x²x
    u = vmul_vd_vd_vd(u, vmul_vd_vd_vd(x2, vd2getx_vd_vd2(x)));

    // Calculate π/2 - (x + u) using double-double arithmetic
    let mut y = ddsub_vd2_vd2_vd2(
        vcast_vd2_d_d(3.141592653589793116 / 2.0, 1.2246467991473532072e-16 / 2.0),
        ddadd_vd2_vd_vd(
            vmulsign_vd_vd_vd(vd2getx_vd_vd2(x), d),
            vmulsign_vd_vd_vd(u, d),
        ),
    );
    x = ddadd_vd2_vd2_vd(x, u);

    // Select result based on input range
    y = vsel_vd2_vo_vd2_vd2(o, y, ddscale_vd2_vd2_vd(x, vcast_vd_d(2.0)));

    // Handle negative inputs
    y = vsel_vd2_vo_vd2_vd2(
        vandnot_vo_vo_vo(o, vlt_vo_vd_vd(d, vcast_vd_d(0.0))),
        ddsub_vd2_vd2_vd2(
            vcast_vd2_d_d(3.141592653589793116, 1.2246467991473532072e-16),
            y,
        ),
        y,
    );

    // Combine high and low parts
    vadd_vd_vd_vd(vd2getx_vd_vd2(y), vd2gety_vd_vd2(y))
}

#[inline(always)]
pub(crate) unsafe fn xatan_u1(d: VDouble) -> VDouble {
    // Calculate atan using atan2k_u1 with denominator of 1
    let d2 = atan2k_u1(
        vcast_vd2_vd_vd(vabs_vd_vd(d), vcast_vd_d(0.0)), // numerator
        vcast_vd2_d_d(1.0, 0.0),                         // denominator
    );

    // Combine high and low parts
    let mut r = vadd_vd_vd_vd(vd2getx_vd_vd2(d2), vd2gety_vd_vd2(d2));

    // Handle infinite inputs
    r = vsel_vd_vo_vd_vd(
        visinf_vo_vd(d),
        vcast_vd_d(1.570796326794896557998982), // π/2
        r,
    );

    // Apply sign of input
    vmulsign_vd_vd_vd(r, d)
}

#[inline(always)]
pub(crate) unsafe fn xlog_u1(d: VDouble) -> VDouble {
    let x: VDouble2;
    let t: VDouble;
    let x2: VDouble;

    // Handle subnormal numbers and extract exponent/mantissa
    #[cfg(not(target_feature = "avx512f"))]
    let (m, e) = {
        let o = vlt_vo_vd_vd(d, vcast_vd_d(SLEEF_DBL_MIN));
        let d = vsel_vd_vo_vd_vd(
            o,
            vmul_vd_vd_vd(d, vcast_vd_d((1u64 << 32) as f64 * (1u64 << 32) as f64)),
            d,
        );
        let mut e = vilogb2k_vi_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        let m = vldexp3_vd_vd_vi(d, vneg_vi_vi(e));
        e = vsel_vi_vo_vi_vi(vcast_vo32_vo64(o), vsub_vi_vi_vi(e, vcast_vi_i(64)), e);
        (m, e)
    };

    #[cfg(target_feature = "avx512f")]
    let (m, e) = {
        let mut e = vgetexp_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        e = vsel_vd_vo_vd_vd(vispinf_vo_vd(e), vcast_vd_d(1024.0), e);
        let m = vgetmant_vd_vd(d);
        (m, e)
    };

    // Calculate main approximation
    x = dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd_vd(vcast_vd_d(-1.0), m),
        ddadd2_vd2_vd_vd(vcast_vd_d(1.0), m),
    );
    x2 = vmul_vd_vd_vd(vd2getx_vd_vd2(x), vd2getx_vd_vd2(x));

    // Polynomial evaluation
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    t = poly7d(
        x2,
        x4,
        x8,
        0.1532076988502701353e+0,
        0.1525629051003428716e+0,
        0.1818605932937785996e+0,
        0.2222214519839380009e+0,
        0.2857142932794299317e+0,
        0.3999999999635251990e+0,
        0.6666666666667333541e+0,
    );

    // Combine with exponent
    #[cfg(not(target_feature = "avx512f"))]
    let mut s = ddmul_vd2_vd2_vd(
        vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
        vcast_vd_vi(e),
    );

    #[cfg(target_feature = "avx512f")]
    let mut s = ddmul_vd2_vd2_vd(
        vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
        e,
    );

    // Final computations
    s = ddadd_vd2_vd2_vd2(s, ddscale_vd2_vd2_vd(x, vcast_vd_d(2.0)));
    s = ddadd_vd2_vd2_vd(s, vmul_vd_vd_vd(vmul_vd_vd_vd(x2, vd2getx_vd_vd2(x)), t));

    let mut r = vadd_vd_vd_vd(vd2getx_vd_vd2(s), vd2gety_vd_vd2(s));

    // Handle special cases
    #[cfg(not(target_feature = "avx512f"))]
    {
        r = vsel_vd_vo_vd_vd(vispinf_vo_vd(d), vcast_vd_d(f64::INFINITY), r);
        r = vsel_vd_vo_vd_vd(
            vor_vo_vo_vo(vlt_vo_vd_vd(d, vcast_vd_d(0.0)), visnan_vo_vd(d)),
            vcast_vd_d(f64::NAN),
            r,
        );
        r = vsel_vd_vo_vd_vd(
            veq_vo_vd_vd(d, vcast_vd_d(0.0)),
            vcast_vd_d(f64::NEG_INFINITY),
            r,
        );
    }

    #[cfg(target_feature = "avx512f")]
    {
        r = vfixup_vd_vd_vd_vi2_i(
            r,
            d,
            vcast_vi2_i((4 << (2 * 4)) | (3 << (4 * 4)) | (5 << (5 * 4)) | (2 << (6 * 4))),
            0,
        );
    }

    r
}

#[inline(always)]
pub(crate) unsafe fn xcbrt_u1(d: VDouble) -> VDouble {
    let mut x: VDouble;
    let mut y: VDouble;
    let mut z: VDouble;
    let t: VDouble;
    let mut q2 = vcast_vd2_d_d(1.0, 0.0);
    let mut u: VDouble2;
    let mut v: VDouble2;
    let e: VInt;
    let qu: VInt;
    let re: VInt;

    // Store original input for AVX512
    #[cfg(target_feature = "avx512f")]
    let s = d;

    // Extract exponent and normalize input
    e = vadd_vi_vi_vi(vilogbk_vi_vd(vabs_vd_vd(d)), vcast_vi_i(1));
    let mut d = vldexp2_vd_vd_vi(d, vneg_vi_vi(e));

    // Calculate quotient and remainder for cube root
    t = vadd_vd_vd_vd(vcast_vd_vi(e), vcast_vd_d(6144.0));
    qu = vtruncate_vi_vd(vmul_vd_vd_vd(t, vcast_vd_d(1.0 / 3.0)));
    re = vtruncate_vi_vd(vsub_vd_vd_vd(
        t,
        vmul_vd_vd_vd(vcast_vd_vi(qu), vcast_vd_d(3.0)),
    ));

    // Select correction factor based on remainder
    q2 = vsel_vd2_vo_vd2_vd2(
        vcast_vo64_vo32(veq_vo_vi_vi(re, vcast_vi_i(1))),
        vcast_vd2_d_d(1.2599210498948731907, -2.5899333753005069177e-17),
        q2,
    );
    q2 = vsel_vd2_vo_vd2_vd2(
        vcast_vo64_vo32(veq_vo_vi_vi(re, vcast_vi_i(2))),
        vcast_vd2_d_d(1.5874010519681995834, -1.0869008194197822986e-16),
        q2,
    );

    // Apply sign to correction factor
    q2 = vd2setxy_vd2_vd_vd(
        vmulsign_vd_vd_vd(vd2getx_vd_vd2(q2), d),
        vmulsign_vd_vd_vd(vd2gety_vd_vd2(q2), d),
    );
    d = vabs_vd_vd(d);

    // Initial approximation using polynomial
    x = vcast_vd_d(-0.640245898480692909870982);
    x = vmla_vd_vd_vd_vd(x, d, vcast_vd_d(2.96155103020039511818595));
    x = vmla_vd_vd_vd_vd(x, d, vcast_vd_d(-5.73353060922947843636166));
    x = vmla_vd_vd_vd_vd(x, d, vcast_vd_d(6.03990368989458747961407));
    x = vmla_vd_vd_vd_vd(x, d, vcast_vd_d(-3.85841935510444988821632));
    x = vmla_vd_vd_vd_vd(x, d, vcast_vd_d(2.2307275302496609725722));

    // First refinement step
    y = vmul_vd_vd_vd(x, x);
    y = vmul_vd_vd_vd(y, y);
    x = vsub_vd_vd_vd(
        x,
        vmul_vd_vd_vd(vmlapn_vd_vd_vd_vd(d, y, x), vcast_vd_d(1.0 / 3.0)),
    );

    z = x;

    // Second refinement step using double-double arithmetic
    u = ddmul_vd2_vd_vd(x, x);
    u = ddmul_vd2_vd2_vd2(u, u);
    u = ddmul_vd2_vd2_vd(u, d);
    u = ddadd2_vd2_vd2_vd(u, vneg_vd_vd(x));
    y = vadd_vd_vd_vd(vd2getx_vd_vd2(u), vd2gety_vd_vd2(u));

    // Final refinement and scaling
    y = vmul_vd_vd_vd(vmul_vd_vd_vd(vcast_vd_d(-2.0 / 3.0), y), z);
    v = ddadd2_vd2_vd2_vd(ddmul_vd2_vd_vd(z, z), y);
    v = ddmul_vd2_vd2_vd(v, d);
    v = ddmul_vd2_vd2_vd2(v, q2);
    z = vldexp2_vd_vd_vi(
        vadd_vd_vd_vd(vd2getx_vd_vd2(v), vd2gety_vd_vd2(v)),
        vsub_vi_vi_vi(qu, vcast_vi_i(2048)),
    );

    // Handle special cases
    #[cfg(not(target_feature = "avx512f"))]
    {
        z = vsel_vd_vo_vd_vd(
            visinf_vo_vd(d),
            vmulsign_vd_vd_vd(vcast_vd_d(f64::INFINITY), vd2getx_vd_vd2(q2)),
            z,
        );
        z = vsel_vd_vo_vd_vd(
            veq_vo_vd_vd(d, vcast_vd_d(0.0)),
            vreinterpret_vd_vm(vsignbit_vm_vd(vd2getx_vd_vd2(q2))),
            z,
        );
    }

    #[cfg(target_feature = "avx512f")]
    {
        z = vsel_vd_vo_vd_vd(
            visinf_vo_vd(s),
            vmulsign_vd_vd_vd(vcast_vd_d(SLEEF_INFINITY), s),
            z,
        );
        z = vsel_vd_vo_vd_vd(
            veq_vo_vd_vd(s, vcast_vd_d(0.0)),
            vmulsign_vd_vd_vd(vcast_vd_d(0.0), s),
            z,
        );
    }

    z
}

#[inline(always)]
pub(crate) unsafe fn xexp(d: VDouble) -> VDouble {
    // Round d/ln(2) to get multiplier for scaling
    let u = vrint_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(R_LN2)));
    let q = vrint_vi_vd(u);

    // Range reduction: s = d - u*ln(2)
    let mut s = vmla_vd_vd_vd_vd(u, vcast_vd_d(-L2U), d);
    s = vmla_vd_vd_vd_vd(u, vcast_vd_d(-L2L), s);

    let mut u: VDouble;

    #[cfg(target_feature = "fma")]
    {
        // Calculate powers of s
        let s2 = vmul_vd_vd_vd(s, s);
        let s4 = vmul_vd_vd_vd(s2, s2);
        let s8 = vmul_vd_vd_vd(s4, s4);

        // Polynomial approximation with FMA
        u = poly10d(
            s,
            s2,
            s4,
            s8,
            0.2081276378237164457e-8,
            0.2511210703042288022e-7,
            0.2755762628169491192e-6,
            0.2755723402025388239e-5,
            0.2480158687479686264e-4,
            0.1984126989855865850e-3,
            0.1388888888914497797e-2,
            0.8333333333314938210e-2,
            0.4166666666666602598e-1,
            0.1666666666666669072e+0
        );

        // Final polynomial terms using FMA
        u = vfma_vd_vd_vd_vd(u, s, vcast_vd_d(0.5000000000000000000e+0));
        u = vfma_vd_vd_vd_vd(u, s, vcast_vd_d(0.1000000000000000000e+1));
        u = vfma_vd_vd_vd_vd(u, s, vcast_vd_d(0.1000000000000000000e+1));
    }

    #[cfg(not(target_feature = "fma"))]
    {
        // Calculate powers of s
        let s2 = vmul_vd_vd_vd(s, s);
        let s4 = vmul_vd_vd_vd(s2, s2);
        let s8 = vmul_vd_vd_vd(s4, s4);

        // Polynomial approximation without FMA
        u = poly10d(
            s,
            s2,
            s4,
            s8,
            2.08860621107283687536341e-09,
            2.51112930892876518610661e-08,
            2.75573911234900471893338e-07,
            2.75572362911928827629423e-06,
            2.4801587159235472998791e-05,
            0.000198412698960509205564975,
            0.00138888888889774492207962,
            0.00833333333331652721664984,
            0.0416666666666665047591422,
            0.166666666666666851703837,
        );

        // Final polynomial terms without FMA
        u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.5000000000000000000e+0));
        u = vadd_vd_vd_vd(vcast_vd_d(1.0), vmla_vd_vd_vd_vd(vmul_vd_vd_vd(s, s), u, s));
    }

    // Scale result by 2^q
    u = vldexp2_vd_vd_vi(u, q);

    // Handle overflow and underflow
    let o = vgt_vo_vd_vd(d, vcast_vd_d(LOG_DBL_MAX));
    u = vsel_vd_vo_vd_vd(o, vcast_vd_d(f64::INFINITY), u);
    u = vreinterpret_vd_vm(vandnot_vm_vo64_vm(
        vlt_vo_vd_vd(d, vcast_vd_d(-1000.0)),
        vreinterpret_vm_vd(u),
    ));

    u
}

#[inline(always)]
unsafe fn logk(d: VDouble) -> VDouble2 {
    let mut x: VDouble2;
    let x2: VDouble2;
    let mut s: VDouble2;
    let t: VDouble;

    // Handle subnormal numbers and extract exponent/mantissa
    #[cfg(not(target_feature = "avx512f"))]
    let (m, e) = {
        let o = vlt_vo_vd_vd(d, vcast_vd_d(SLEEF_DBL_MIN));
        let d = vsel_vd_vo_vd_vd(
            o,
            vmul_vd_vd_vd(d, vcast_vd_d((1u64 << 32) as f64 * (1u64 << 32) as f64)),
            d,
        );
        let mut e = vilogb2k_vi_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        let m = vldexp3_vd_vd_vi(d, vneg_vi_vi(e));
        e = vsel_vi_vo_vi_vi(vcast_vo32_vo64(o), vsub_vi_vi_vi(e, vcast_vi_i(64)), e);
        (m, e)
    };

    #[cfg(target_feature = "avx512f")]
    let (m, e) = {
        let mut e = vgetexp_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        e = vsel_vd_vo_vd_vd(vispinf_vo_vd(e), vcast_vd_d(1024.0), e);
        let m = vgetmant_vd_vd(d);
        (m, e)
    };

    // Calculate main approximation
    x = dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd_vd(vcast_vd_d(-1.0), m),
        ddadd2_vd2_vd_vd(vcast_vd_d(1.0), m),
    );
    x2 = ddsqu_vd2_vd2(x);

    // Polynomial evaluation
    let x4 = vmul_vd_vd_vd(vd2getx_vd_vd2(x2), vd2getx_vd_vd2(x2));
    let x8 = vmul_vd_vd_vd(x4, x4);
    let x16 = vmul_vd_vd_vd(x8, x8);
    t = poly9d(
        vd2getx_vd_vd2(x2),
        x4,
        x8,
        x16,
        0.116255524079935043668677,
        0.103239680901072952701192,
        0.117754809412463995466069,
        0.13332981086846273921509,
        0.153846227114512262845736,
        0.181818180850050775676507,
        0.222222222230083560345903,
        0.285714285714249172087875,
        0.400000000000000077715612,
    );

    // Final computations
    let c = vcast_vd2_d_d(0.666666666666666629659233, 3.80554962542412056336616e-17);

    #[cfg(not(target_feature = "avx512f"))]
    {
        s = ddmul_vd2_vd2_vd(
            vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
            vcast_vd_vi(e),
        );
    }

    #[cfg(target_feature = "avx512f")]
    {
        s = ddmul_vd2_vd2_vd(
            vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
            e,
        );
    }

    s = ddadd_vd2_vd2_vd2(s, ddscale_vd2_vd2_vd(x, vcast_vd_d(2.0)));
    x = ddmul_vd2_vd2_vd2(x2, x);
    s = ddadd_vd2_vd2_vd2(s, ddmul_vd2_vd2_vd2(x, c));
    x = ddmul_vd2_vd2_vd2(x2, x);
    s = ddadd_vd2_vd2_vd2(s, ddmul_vd2_vd2_vd(x, t));

    s
}

#[inline(always)]
unsafe fn expk(d: VDouble2) -> VDouble {
    // Calculate q = round(d/ln(2))
    let u = vmul_vd_vd_vd(
        vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d)),
        vcast_vd_d(R_LN2),
    );
    let dq = vrint_vd_vd(u);
    let q = vrint_vi_vd(dq);
    let mut s: VDouble2;
    let mut t: VDouble2;

    // Range reduction: s = d - q*ln(2)
    s = ddadd2_vd2_vd2_vd(d, vmul_vd_vd_vd(dq, vcast_vd_d(-L2U)));
    s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dq, vcast_vd_d(-L2L)));

    // Normalize the reduced range
    s = ddnormalize_vd2_vd2(s);

    // Calculate powers of s
    let s2 = vmul_vd_vd_vd(vd2getx_vd_vd2(s), vd2getx_vd_vd2(s));
    let s4 = vmul_vd_vd_vd(s2, s2);
    let s8 = vmul_vd_vd_vd(s4, s4);

    // Polynomial approximation
    let mut u = poly10d(
        vd2getx_vd_vd2(s),
        s2,
        s4,
        s8,
        2.51069683420950419527139e-08,
        2.76286166770270649116855e-07,
        2.75572496725023574143864e-06,
        2.48014973989819794114153e-05,
        0.000198412698809069797676111,
        0.0013888888939977128960529,
        0.00833333333332371417601081,
        0.0416666666665409524128449,
        0.166666666666666740681535,
        0.500000000000000999200722,
    );

    // Combine polynomial terms
    t = ddadd_vd2_vd_vd2(vcast_vd_d(1.0), s);
    t = ddadd_vd2_vd2_vd2(t, ddmul_vd2_vd2_vd(ddsqu_vd2_vd2(s), u));

    // Combine high and low parts
    u = vadd_vd_vd_vd(vd2getx_vd_vd2(t), vd2gety_vd_vd2(t));

    // Scale by 2^q
    u = vldexp2_vd_vd_vi(u, q);

    // Handle underflow
    u = vreinterpret_vd_vm(vandnot_vm_vo64_vm(
        vlt_vo_vd_vd(vd2getx_vd_vd2(d), vcast_vd_d(-1000.0)),
        vreinterpret_vm_vd(u),
    ));

    u
}

#[inline(always)]
pub(crate) unsafe fn xpow(x: VDouble, y: VDouble) -> VDouble {
    // Check if y is an integer and if it's odd
    let yisint = visint_vo_vd(y);
    let yisodd = vand_vo_vo_vo(visodd_vo_vd(y), yisint);

    // Calculate log(x) * y
    let d = ddmul_vd2_vd2_vd(logk(vabs_vd_vd(x)), y);
    let mut result = expk(d);

    // Handle overflow
    let o = vgt_vo_vd_vd(vd2getx_vd_vd2(d), vcast_vd_d(LOG_DBL_MAX));
    result = vsel_vd_vo_vd_vd(o, vcast_vd_d(f64::INFINITY), result);

    // Adjust result for negative x
    result = vmul_vd_vd_vd(
        result,
        vsel_vd_vo_vd_vd(
            vgt_vo_vd_vd(x, vcast_vd_d(0.0)),
            vcast_vd_d(1.0),
            vsel_vd_vo_vd_vd(
                yisint,
                vsel_vd_vo_vd_vd(yisodd, vcast_vd_d(-1.0), vcast_vd_d(1.0)),
                vcast_vd_d(f64::NAN),
            ),
        ),
    );

    // Handle infinite y
    let efx = vmulsign_vd_vd_vd(vsub_vd_vd_vd(vabs_vd_vd(x), vcast_vd_d(1.0)), y);
    result = vsel_vd_vo_vd_vd(
        visinf_vo_vd(y),
        vreinterpret_vd_vm(vandnot_vm_vo64_vm(
            vlt_vo_vd_vd(efx, vcast_vd_d(0.0)),
            vreinterpret_vm_vd(vsel_vd_vo_vd_vd(
                veq_vo_vd_vd(efx, vcast_vd_d(0.0)),
                vcast_vd_d(1.0),
                vcast_vd_d(f64::INFINITY),
            )),
        )),
        result,
    );

    // Handle x = 0 or x = infinity
    result = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(visinf_vo_vd(x), veq_vo_vd_vd(x, vcast_vd_d(0.0))),
        vmulsign_vd_vd_vd(
            vsel_vd_vo_vd_vd(
                vxor_vo_vo_vo(vsignbit_vo_vd(y), veq_vo_vd_vd(x, vcast_vd_d(0.0))),
                vcast_vd_d(0.0),
                vcast_vd_d(f64::INFINITY),
            ),
            vsel_vd_vo_vd_vd(yisodd, x, vcast_vd_d(1.0)),
        ),
        result,
    );

    // Handle NaN inputs
    result = vreinterpret_vd_vm(vor_vm_vo64_vm(
        vor_vo_vo_vo(visnan_vo_vd(x), visnan_vo_vd(y)),
        vreinterpret_vm_vd(result),
    ));

    // Handle y = 0 or x = 1
    result = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            veq_vo_vd_vd(y, vcast_vd_d(0.0)),
            veq_vo_vd_vd(x, vcast_vd_d(1.0)),
        ),
        vcast_vd_d(1.0),
        result,
    );

    result
}

#[inline(always)]
unsafe fn expk2(d: VDouble2) -> VDouble2 {
    // Calculate q = round(d/ln(2))
    let u = vmul_vd_vd_vd(
        vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d)),
        vcast_vd_d(R_LN2),
    );
    let dq = vrint_vd_vd(u);
    let q = vrint_vi_vd(dq);
    let mut s: VDouble2;
    let mut t: VDouble2;

    // Range reduction: s = d - q*ln(2)
    s = ddadd2_vd2_vd2_vd(d, vmul_vd_vd_vd(dq, vcast_vd_d(-L2U)));
    s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(dq, vcast_vd_d(-L2L)));

    // Calculate powers using double-double arithmetic
    let s2 = ddsqu_vd2_vd2(s);
    let s4 = ddsqu_vd2_vd2(s2);
    let s8 = vmul_vd_vd_vd(vd2getx_vd_vd2(s4), vd2getx_vd_vd2(s4));

    // Polynomial approximation
    let u = poly10d(
        vd2getx_vd_vd2(s),
        vd2getx_vd_vd2(s2),
        vd2getx_vd_vd2(s4),
        s8,
        0.1602472219709932072e-9,
        0.2092255183563157007e-8,
        0.2505230023782644465e-7,
        0.2755724800902135303e-6,
        0.2755731892386044373e-5,
        0.2480158735605815065e-4,
        0.1984126984148071858e-3,
        0.1388888888886763255e-2,
        0.8333333333333347095e-2,
        0.4166666666666669905e-1,
    );

    // Combine polynomial terms using double-double arithmetic
    t = ddadd_vd2_vd_vd2(
        vcast_vd_d(0.5),
        ddmul_vd2_vd2_vd(s, vcast_vd_d(0.1666666666666666574e+0)),
    );
    t = ddadd_vd2_vd_vd2(vcast_vd_d(1.0), ddmul_vd2_vd2_vd2(t, s));
    t = ddadd_vd2_vd_vd2(vcast_vd_d(1.0), ddmul_vd2_vd2_vd2(t, s));
    t = ddadd_vd2_vd2_vd2(t, ddmul_vd2_vd2_vd(s4, u));

    // Scale both parts by 2^q
    t = vd2setx_vd2_vd2_vd(t, vldexp2_vd_vd_vi(vd2getx_vd_vd2(t), q));
    t = vd2sety_vd2_vd2_vd(t, vldexp2_vd_vd_vi(vd2gety_vd_vd2(t), q));

    // Handle underflow for both parts
    t = vd2setx_vd2_vd2_vd(
        t,
        vreinterpret_vd_vm(vandnot_vm_vo64_vm(
            vlt_vo_vd_vd(vd2getx_vd_vd2(d), vcast_vd_d(-1000.0)),
            vreinterpret_vm_vd(vd2getx_vd_vd2(t)),
        )),
    );
    t = vd2sety_vd2_vd2_vd(
        t,
        vreinterpret_vd_vm(vandnot_vm_vo64_vm(
            vlt_vo_vd_vd(vd2getx_vd_vd2(d), vcast_vd_d(-1000.0)),
            vreinterpret_vm_vd(vd2gety_vd_vd2(t)),
        )),
    );

    t
}

#[inline(always)]
pub(crate) unsafe fn xsinh(x: VDouble) -> VDouble {
    // Get absolute value of input
    let mut y = vabs_vd_vd(x);

    // Calculate exp(x) using double-double arithmetic
    let mut d = expk2(vcast_vd2_vd_vd(y, vcast_vd_d(0.0)));

    // Calculate sinh(x) = (exp(x) - 1/exp(x))/2
    d = ddsub_vd2_vd2_vd2(d, ddrec_vd2_vd2(d));
    y = vmul_vd_vd_vd(
        vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d)),
        vcast_vd_d(0.5),
    );

    // Handle overflow and NaN
    y = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            vgt_vo_vd_vd(vabs_vd_vd(x), vcast_vd_d(710.0)),
            visnan_vo_vd(y),
        ),
        vcast_vd_d(f64::INFINITY),
        y,
    );

    // Apply sign of input
    y = vmulsign_vd_vd_vd(y, x);

    // Propagate NaN
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(visnan_vo_vd(x), vreinterpret_vm_vd(y)));

    y
}

#[inline(always)]
pub(crate) unsafe fn xcosh(x: VDouble) -> VDouble {
    // Get absolute value of input
    let mut y = vabs_vd_vd(x);

    // Calculate exp(x) using double-double arithmetic
    let mut d = expk2(vcast_vd2_vd_vd(y, vcast_vd_d(0.0)));

    // Calculate cosh(x) = (exp(x) + 1/exp(x))/2
    d = ddadd_vd2_vd2_vd2(d, ddrec_vd2_vd2(d));
    y = vmul_vd_vd_vd(
        vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d)),
        vcast_vd_d(0.5),
    );

    // Handle overflow and NaN
    y = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            vgt_vo_vd_vd(vabs_vd_vd(x), vcast_vd_d(710.0)),
            visnan_vo_vd(y),
        ),
        vcast_vd_d(f64::INFINITY),
        y,
    );

    // Propagate NaN
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(visnan_vo_vd(x), vreinterpret_vm_vd(y)));

    y
}

#[inline(always)]
pub(crate) unsafe fn xtanh(x: VDouble) -> VDouble {
    // Get absolute value of input
    let mut y = vabs_vd_vd(x);

    // Calculate exp(x) using double-double arithmetic
    let d = expk2(vcast_vd2_vd_vd(y, vcast_vd_d(0.0)));

    // Calculate 1/exp(x)
    let e = ddrec_vd2_vd2(d);

    // Calculate tanh(x) = (exp(x) - 1/exp(x))/(exp(x) + 1/exp(x))
    let d = dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd2_vd2(d, ddneg_vd2_vd2(e)),
        ddadd2_vd2_vd2_vd2(d, e),
    );
    y = vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d));

    // Handle overflow (approaches ±1) and NaN
    y = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            vgt_vo_vd_vd(vabs_vd_vd(x), vcast_vd_d(18.714973875)),
            visnan_vo_vd(y),
        ),
        vcast_vd_d(1.0),
        y,
    );

    // Apply sign of input
    y = vmulsign_vd_vd_vd(y, x);

    // Propagate NaN
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(visnan_vo_vd(x), vreinterpret_vm_vd(y)));

    y
}

#[inline(always)]
unsafe fn logk2(d: VDouble2) -> VDouble2 {
    let x: VDouble2;
    let x2: VDouble2;
    let m: VDouble2;
    let mut s: VDouble2;
    let mut t: VDouble;

    // Extract exponent
    let e = vilogbk_vi_vd(vmul_vd_vd_vd(vd2getx_vd_vd2(d), vcast_vd_d(1.0 / 0.75)));

    // Normalize mantissa
    m = vd2setxy_vd2_vd_vd(
        vldexp2_vd_vd_vi(vd2getx_vd_vd2(d), vneg_vi_vi(e)),
        vldexp2_vd_vd_vi(vd2gety_vd_vd2(d), vneg_vi_vi(e)),
    );

    // Calculate (m-1)/(m+1)
    x = dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd2_vd(m, vcast_vd_d(-1.0)),
        ddadd2_vd2_vd2_vd(m, vcast_vd_d(1.0)),
    );
    x2 = ddsqu_vd2_vd2(x);

    // Polynomial approximation
    let x4 = vmul_vd_vd_vd(vd2getx_vd_vd2(x2), vd2getx_vd_vd2(x2));
    let x8 = vmul_vd_vd_vd(x4, x4);
    t = poly7d(
        vd2getx_vd_vd2(x2),
        x4,
        x8,
        0.13860436390467167910856,
        0.131699838841615374240845,
        0.153914168346271945653214,
        0.181816523941564611721589,
        0.22222224632662035403996,
        0.285714285511134091777308,
        0.400000000000914013309483,
    );
    t = vmla_vd_vd_vd_vd(
        t,
        vd2getx_vd_vd2(x2),
        vcast_vd_d(0.666666666666664853302393),
    );

    // Combine terms
    s = ddmul_vd2_vd2_vd(
        vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
        vcast_vd_vi(e),
    );
    s = ddadd_vd2_vd2_vd2(s, ddscale_vd2_vd2_vd(x, vcast_vd_d(2.0)));
    s = ddadd_vd2_vd2_vd2(s, ddmul_vd2_vd2_vd(ddmul_vd2_vd2_vd2(x2, x), t));

    s
}

#[inline(always)]
pub(crate) unsafe fn xasinh(x: VDouble) -> VDouble {
    // Get absolute value of input
    let mut y = vabs_vd_vd(x);

    // Check if |x| > 1
    let o = vgt_vo_vd_vd(y, vcast_vd_d(1.0));
    let mut d: VDouble2;

    // Select computation path based on magnitude
    d = vsel_vd2_vo_vd2_vd2(
        o,
        ddrec_vd2_vd(x),                     // if |x| > 1: 1/x
        vcast_vd2_vd_vd(y, vcast_vd_d(0.0)), // if |x| ≤ 1: |x|
    );

    // Calculate sqrt(d^2 + 1)
    d = ddsqrt_vd2_vd2(ddadd2_vd2_vd2_vd(ddsqu_vd2_vd2(d), vcast_vd_d(1.0)));

    // If |x| > 1, multiply by |x|
    d = vsel_vd2_vo_vd2_vd2(o, ddmul_vd2_vd2_vd(d, y), d);

    // Calculate log(d + x)
    d = logk2(ddnormalize_vd2_vd2(ddadd2_vd2_vd2_vd(d, x)));
    y = vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d));

    // Handle overflow and NaN
    y = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            vgt_vo_vd_vd(vabs_vd_vd(x), vcast_vd_d(SQRT_DBL_MAX)),
            visnan_vo_vd(y),
        ),
        vmulsign_vd_vd_vd(vcast_vd_d(f64::INFINITY), x),
        y,
    );

    // Propagate NaN
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(visnan_vo_vd(x), vreinterpret_vm_vd(y)));

    // Handle negative zero
    y = vsel_vd_vo_vd_vd(visnegzero_vo_vd(x), vcast_vd_d(-0.0), y);

    y
}

#[inline(always)]
pub(crate) unsafe fn xacosh(x: VDouble) -> VDouble {
    // Calculate acosh using log: acosh(x) = log(x + sqrt((x+1)(x-1)))
    let d = logk2(ddadd2_vd2_vd2_vd(
        ddmul_vd2_vd2_vd2(
            ddsqrt_vd2_vd2(ddadd2_vd2_vd_vd(x, vcast_vd_d(1.0))),
            ddsqrt_vd2_vd2(ddadd2_vd2_vd_vd(x, vcast_vd_d(-1.0))),
        ),
        x,
    ));
    let mut y = vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d));

    // Handle overflow and NaN
    y = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            vgt_vo_vd_vd(vabs_vd_vd(x), vcast_vd_d(SQRT_DBL_MAX)),
            visnan_vo_vd(y),
        ),
        vcast_vd_d(f64::INFINITY),
        y,
    );

    // Handle x = 1 (acosh(1) = 0)
    y = vreinterpret_vd_vm(vandnot_vm_vo64_vm(
        veq_vo_vd_vd(x, vcast_vd_d(1.0)),
        vreinterpret_vm_vd(y),
    ));

    // Return NaN for x < 1 (domain error)
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(
        vlt_vo_vd_vd(x, vcast_vd_d(1.0)),
        vreinterpret_vm_vd(y),
    ));

    // Propagate NaN input
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(visnan_vo_vd(x), vreinterpret_vm_vd(y)));

    y
}

#[inline(always)]
pub(crate) unsafe fn xatanh(x: VDouble) -> VDouble {
    // Get absolute value of input
    let y = vabs_vd_vd(x);

    // Calculate atanh using log: atanh(x) = 0.5 * log((1+x)/(1-x))
    let d = logk2(dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd_vd(vcast_vd_d(1.0), y),
        ddadd2_vd2_vd_vd(vcast_vd_d(1.0), vneg_vd_vd(y)),
    ));

    // Handle domain and special cases
    let mut y = vreinterpret_vd_vm(vor_vm_vo64_vm(
        vgt_vo_vd_vd(y, vcast_vd_d(1.0)),
        vreinterpret_vm_vd(vsel_vd_vo_vd_vd(
            veq_vo_vd_vd(y, vcast_vd_d(1.0)),
            vcast_vd_d(f64::INFINITY),
            vmul_vd_vd_vd(
                vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d)),
                vcast_vd_d(0.5),
            ),
        )),
    ));

    // Apply sign of input
    y = vmulsign_vd_vd_vd(y, x);

    // Handle infinite inputs
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(
        vor_vo_vo_vo(visinf_vo_vd(x), visnan_vo_vd(y)),
        vreinterpret_vm_vd(y),
    ));

    // Propagate NaN
    y = vreinterpret_vd_vm(vor_vm_vo64_vm(visnan_vo_vd(x), vreinterpret_vm_vd(y)));

    y
}

#[inline(always)]
pub(crate) unsafe fn xexp2(d: VDouble) -> VDouble {
    // Round to nearest integer and get fractional part
    let u = vrint_vd_vd(d);
    let s = vsub_vd_vd_vd(d, u);
    let q = vrint_vi_vd(u);

    // Calculate powers of s for polynomial
    let s2 = vmul_vd_vd_vd(s, s);
    let s4 = vmul_vd_vd_vd(s2, s2);
    let s8 = vmul_vd_vd_vd(s4, s4);

    // Polynomial approximation
    let mut u = poly10d(
        s,
        s2,
        s4,
        s8,
        0.4434359082926529454e-9,
        0.7073164598085707425e-8,
        0.1017819260921760451e-6,
        0.1321543872511327615e-5,
        0.1525273353517584730e-4,
        0.1540353045101147808e-3,
        0.1333355814670499073e-2,
        0.9618129107597600536e-2,
        0.5550410866482046596e-1,
        0.2402265069591012214e+0,
    );

    // Add ln(2) term
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.6931471805599452862e+0));

    // Final computation differs based on FMA support
    #[cfg(target_feature = "fma")]
    {
        u = vfma_vd_vd_vd_vd(u, s, vcast_vd_d(1.0));
    }

    #[cfg(not(target_feature = "fma"))]
    {
        u = vd2getx_vd_vd2(ddnormalize_vd2_vd2(ddadd_vd2_vd_vd2(
            vcast_vd_d(1.0),
            ddmul_vd2_vd_vd(u, s),
        )));
    }

    // Scale by 2^q
    u = vldexp2_vd_vd_vi(u, q);

    // Handle special cases
    u = vsel_vd_vo_vd_vd(
        vge_vo_vd_vd(d, vcast_vd_d(1024.0)),
        vcast_vd_d(f64::INFINITY),
        u,
    );
    u = vreinterpret_vd_vm(vandnot_vm_vo64_vm(
        vlt_vo_vd_vd(d, vcast_vd_d(-2000.0)),
        vreinterpret_vm_vd(u),
    ));

    u
}

#[inline(always)]
pub(crate) unsafe fn xexp10(d: VDouble) -> VDouble {
    // Calculate q = round(d * log10(2))
    let u = vrint_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(LOG10_2)));
    let q = vrint_vi_vd(u);

    // Range reduction: s = d - u * log10(e)
    let mut s = vmla_vd_vd_vd_vd(u, vcast_vd_d(-L10_U), d);
    s = vmla_vd_vd_vd_vd(u, vcast_vd_d(-L10_L), s);

    // Polynomial approximation of exp10(s)
    let mut u = vcast_vd_d(0.2411463498334267652e-3);
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.1157488415217187375e-2));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.5013975546789733659e-2));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.1959762320720533080e-1));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.6808936399446784138e-1));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.2069958494722676234e+0));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.5393829292058536229e+0));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.1171255148908541655e+1));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.2034678592293432953e+1));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.2650949055239205876e+1));
    u = vmla_vd_vd_vd_vd(u, s, vcast_vd_d(0.2302585092994045901e+1));

    // Combine terms
    #[cfg(target_feature = "fma")]
    {
        u = vfma_vd_vd_vd_vd(u, s, vcast_vd_d(1.0));
    }
    #[cfg(not(target_feature = "fma"))]
    {
        u = vd2getx_vd_vd2(ddnormalize_vd2_vd2(ddadd_vd2_vd_vd2(
            vcast_vd_d(1.0),
            ddmul_vd2_vd_vd(u, s),
        )));
    }

    // Scale by 2^q
    u = vldexp2_vd_vd_vi(u, q);

    // Handle overflow and underflow
    u = vsel_vd_vo_vd_vd(
        vgt_vo_vd_vd(d, vcast_vd_d(308.25471555991671)),
        vcast_vd_d(f64::INFINITY),
        u,
    );
    u = vreinterpret_vd_vm(vandnot_vm_vo64_vm(
        vlt_vo_vd_vd(d, vcast_vd_d(-350.0)),
        vreinterpret_vm_vd(u),
    ));

    u
}

#[inline(always)]
pub(crate) unsafe fn xexpm1(a: VDouble) -> VDouble {
    // Calculate exp(a) - 1 using double-double arithmetic
    let d = ddadd2_vd2_vd2_vd(expk2(vcast_vd2_vd_vd(a, vcast_vd_d(0.0))), vcast_vd_d(-1.0));
    let mut x = vadd_vd_vd_vd(vd2getx_vd_vd2(d), vd2gety_vd_vd2(d));

    // Handle overflow
    x = vsel_vd_vo_vd_vd(
        vgt_vo_vd_vd(a, vcast_vd_d(709.782712893383996732223)),
        vcast_vd_d(f64::INFINITY),
        x,
    );

    // Handle underflow
    x = vsel_vd_vo_vd_vd(
        vlt_vo_vd_vd(a, vcast_vd_d(-36.736800569677101399113302437)),
        vcast_vd_d(-1.0),
        x,
    );

    // Handle negative zero
    x = vsel_vd_vo_vd_vd(visnegzero_vo_vd(a), vcast_vd_d(-0.0), x);

    x
}

#[inline(always)]
pub(crate) unsafe fn xlog10(d: VDouble) -> VDouble {
    let x: VDouble2;
    let t: VDouble;
    let x2: VDouble;

    // Handle subnormal numbers and extract exponent/mantissa
    #[cfg(not(target_feature = "avx512f"))]
    let (m, e) = {
        let o = vlt_vo_vd_vd(d, vcast_vd_d(SLEEF_DBL_MIN));
        let d = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(d, vcast_vd_d(2_f64.powi(64))), d);
        let mut e = vilogb2k_vi_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        let m = vldexp3_vd_vd_vi(d, vneg_vi_vi(e));
        e = vsel_vi_vo_vi_vi(vcast_vo32_vo64(o), vsub_vi_vi_vi(e, vcast_vi_i(64)), e);
        (m, e)
    };

    #[cfg(target_feature = "avx512f")]
    let (m, e) = {
        let mut e = vgetexp_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        e = vsel_vd_vo_vd_vd(vispinf_vo_vd(e), vcast_vd_d(1024.0), e);
        let m = vgetmant_vd_vd(d);
        (m, e)
    };

    // Calculate (m-1)/(m+1) and its square
    x = dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd_vd(vcast_vd_d(-1.0), m),
        ddadd2_vd2_vd_vd(vcast_vd_d(1.0), m),
    );
    x2 = vmul_vd_vd_vd(vd2getx_vd_vd2(x), vd2getx_vd_vd2(x));

    // Polynomial approximation
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    t = poly7d(
        x2,
        x4,
        x8,
        0.6653725819576758460e-1,
        0.6625722782820833712e-1,
        0.7898105214313944078e-1,
        0.9650955035715275132e-1,
        0.1240841409721444993e+0,
        0.1737177927454605086e+0,
        0.2895296546021972617e+0,
    );

    // Combine terms
    #[cfg(not(target_feature = "avx512f"))]
    let mut s = ddmul_vd2_vd2_vd(
        vcast_vd2_d_d(0.30102999566398119802, -2.803728127785170339e-18),
        vcast_vd_vi(e),
    );

    #[cfg(target_feature = "avx512f")]
    let mut s = ddmul_vd2_vd2_vd(
        vcast_vd2_d_d(0.30102999566398119802, -2.803728127785170339e-18),
        e,
    );

    s = ddadd_vd2_vd2_vd2(
        s,
        ddmul_vd2_vd2_vd2(
            x,
            vcast_vd2_d_d(0.86858896380650363334, 1.1430059694096389311e-17),
        ),
    );
    s = ddadd_vd2_vd2_vd(s, vmul_vd_vd_vd(vmul_vd_vd_vd(x2, vd2getx_vd_vd2(x)), t));

    let mut r = vadd_vd_vd_vd(vd2getx_vd_vd2(s), vd2gety_vd_vd2(s));

    // Handle special cases
    #[cfg(not(target_feature = "avx512f"))]
    {
        r = vsel_vd_vo_vd_vd(vispinf_vo_vd(d), vcast_vd_d(f64::INFINITY), r);
        r = vsel_vd_vo_vd_vd(
            vor_vo_vo_vo(vlt_vo_vd_vd(d, vcast_vd_d(0.0)), visnan_vo_vd(d)),
            vcast_vd_d(f64::NAN),
            r,
        );
        r = vsel_vd_vo_vd_vd(
            veq_vo_vd_vd(d, vcast_vd_d(0.0)),
            vcast_vd_d(f64::NEG_INFINITY),
            r,
        );
    }

    #[cfg(target_feature = "avx512f")]
    {
        r = vfixup_vd_vd_vd_vi2_i(
            r,
            d,
            vcast_vi2_i((4 << (2 * 4)) | (3 << (4 * 4)) | (5 << (5 * 4)) | (2 << (6 * 4))),
            0,
        );
    }

    r
}

#[inline(always)]
pub(crate) unsafe fn xlog2(d: VDouble) -> VDouble {
    let x: VDouble2;
    let t: VDouble;
    let x2: VDouble;

    // Handle subnormal numbers and extract exponent/mantissa
    #[cfg(not(target_feature = "avx512f"))]
    let (m, e) = {
        let o = vlt_vo_vd_vd(d, vcast_vd_d(SLEEF_DBL_MIN));
        let d = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(d, vcast_vd_d(2_f64.powi(64))), d);
        let mut e = vilogb2k_vi_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        let m = vldexp3_vd_vd_vi(d, vneg_vi_vi(e));
        e = vsel_vi_vo_vi_vi(vcast_vo32_vo64(o), vsub_vi_vi_vi(e, vcast_vi_i(64)), e);
        (m, e)
    };

    #[cfg(target_feature = "avx512f")]
    let (m, e) = {
        let mut e = vgetexp_vd_vd(vmul_vd_vd_vd(d, vcast_vd_d(1.0 / 0.75)));
        e = vsel_vd_vo_vd_vd(vispinf_vo_vd(e), vcast_vd_d(1024.0), e);
        let m = vgetmant_vd_vd(d);
        (m, e)
    };

    // Calculate (m-1)/(m+1) and its square
    x = dddiv_vd2_vd2_vd2(
        ddadd2_vd2_vd_vd(vcast_vd_d(-1.0), m),
        ddadd2_vd2_vd_vd(vcast_vd_d(1.0), m),
    );
    x2 = vmul_vd_vd_vd(vd2getx_vd_vd2(x), vd2getx_vd_vd2(x));

    // Polynomial approximation
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    t = poly7d(
        x2,
        x4,
        x8,
        0.2211941750456081490e+0,
        0.2200768693152277689e+0,
        0.2623708057488514656e+0,
        0.3205977477944495502e+0,
        0.4121985945485324709e+0,
        0.5770780162997058982e+0,
        0.96179669392608091449,
    );

    // Combine terms
    #[cfg(not(target_feature = "avx512f"))]
    let mut s = ddadd2_vd2_vd_vd2(
        vcast_vd_vi(e),
        ddmul_vd2_vd2_vd2(
            x,
            vcast_vd2_d_d(2.885390081777926774, 6.0561604995516736434e-18),
        ),
    );

    #[cfg(target_feature = "avx512f")]
    let mut s = ddadd2_vd2_vd_vd2(
        e,
        ddmul_vd2_vd2_vd2(
            x,
            vcast_vd2_d_d(2.885390081777926774, 6.0561604995516736434e-18),
        ),
    );

    s = ddadd2_vd2_vd2_vd(s, vmul_vd_vd_vd(vmul_vd_vd_vd(x2, vd2getx_vd_vd2(x)), t));

    let mut r = vadd_vd_vd_vd(vd2getx_vd_vd2(s), vd2gety_vd_vd2(s));

    // Handle special cases
    #[cfg(not(target_feature = "avx512f"))]
    {
        r = vsel_vd_vo_vd_vd(vispinf_vo_vd(d), vcast_vd_d(f64::INFINITY), r);
        r = vsel_vd_vo_vd_vd(
            vor_vo_vo_vo(vlt_vo_vd_vd(d, vcast_vd_d(0.0)), visnan_vo_vd(d)),
            vcast_vd_d(f64::NAN),
            r,
        );
        r = vsel_vd_vo_vd_vd(
            veq_vo_vd_vd(d, vcast_vd_d(0.0)),
            vcast_vd_d(f64::NEG_INFINITY),
            r,
        );
    }

    #[cfg(target_feature = "avx512f")]
    {
        r = vfixup_vd_vd_vd_vi2_i(
            r,
            d,
            vcast_vi2_i((4 << (2 * 4)) | (3 << (4 * 4)) | (5 << (5 * 4)) | (2 << (6 * 4))),
            0,
        );
    }

    r
}

#[inline(always)]
pub(crate) unsafe fn xlog1p(d: VDouble) -> VDouble {
    let x: VDouble2;
    let t: VDouble;
    let x2: VDouble;

    let dp1 = vadd_vd_vd_vd(d, vcast_vd_d(1.0));

    // Handle subnormal numbers and extract exponent/mantissa
    #[cfg(not(target_feature = "avx512f"))]
    let (m, s) = {
        let o = vlt_vo_vd_vd(dp1, vcast_vd_d(SLEEF_DBL_MIN));
        let dp1 = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(dp1, vcast_vd_d(2_f64.powi(64))), dp1);
        let mut e = vilogb2k_vi_vd(vmul_vd_vd_vd(dp1, vcast_vd_d(1.0 / 0.75)));
        let t = vldexp3_vd_vd_vi(vcast_vd_d(1.0), vneg_vi_vi(e));
        let m = vmla_vd_vd_vd_vd(d, t, vsub_vd_vd_vd(t, vcast_vd_d(1.0)));
        e = vsel_vi_vo_vi_vi(vcast_vo32_vo64(o), vsub_vi_vi_vi(e, vcast_vi_i(64)), e);
        let s = ddmul_vd2_vd2_vd(
            vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
            vcast_vd_vi(e),
        );
        (m, s)
    };

    #[cfg(target_feature = "avx512f")]
    let (m, s) = {
        let mut e = vgetexp_vd_vd(vmul_vd_vd_vd(dp1, vcast_vd_d(1.0 / 0.75)));
        e = vsel_vd_vo_vd_vd(vispinf_vo_vd(e), vcast_vd_d(1024.0), e);
        let t = vldexp3_vd_vd_vi(vcast_vd_d(1.0), vneg_vi_vi(vrint_vi_vd(e)));
        let m = vmla_vd_vd_vd_vd(d, t, vsub_vd_vd_vd(t, vcast_vd_d(1.0)));
        let s = ddmul_vd2_vd2_vd(
            vcast_vd2_d_d(0.693147180559945286226764, 2.319046813846299558417771e-17),
            e,
        );
        (m, s)
    };

    // Calculate x/(2+x) and its square
    x = dddiv_vd2_vd2_vd2(
        vcast_vd2_vd_vd(m, vcast_vd_d(0.0)),
        ddadd_vd2_vd_vd(vcast_vd_d(2.0), m),
    );
    x2 = vmul_vd_vd_vd(vd2getx_vd_vd2(x), vd2getx_vd_vd2(x));

    // Polynomial approximation
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    t = poly7d(
        x2,
        x4,
        x8,
        0.1532076988502701353e+0,
        0.1525629051003428716e+0,
        0.1818605932937785996e+0,
        0.2222214519839380009e+0,
        0.2857142932794299317e+0,
        0.3999999999635251990e+0,
        0.6666666666667333541e+0,
    );

    // Combine terms
    let mut s = ddadd_vd2_vd2_vd2(s, ddscale_vd2_vd2_vd(x, vcast_vd_d(2.0)));
    s = ddadd_vd2_vd2_vd(s, vmul_vd_vd_vd(vmul_vd_vd_vd(x2, vd2getx_vd_vd2(x)), t));

    let mut r = vadd_vd_vd_vd(vd2getx_vd_vd2(s), vd2gety_vd_vd2(s));

    // Use log(d) if d too large
    let ocore = vle_vo_vd_vd(d, vcast_vd_d(LOG1P_BOUND));
    if vtestallones_i_vo64(ocore) == 0 {
        r = vsel_vd_vo_vd_vd(ocore, r, xlog_u1(d));
    }

    // Handle special cases
    r = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(vlt_vo_vd_vd(d, vcast_vd_d(-1.0)), visnan_vo_vd(d)),
        vcast_vd_d(f64::NAN),
        r,
    );
    r = vsel_vd_vo_vd_vd(
        veq_vo_vd_vd(d, vcast_vd_d(-1.0)),
        vcast_vd_d(f64::NEG_INFINITY),
        r,
    );
    r = vsel_vd_vo_vd_vd(visnegzero_vo_vd(d), vcast_vd_d(-0.0), r);

    r
}

#[inline(always)]
pub(crate) unsafe fn xsqrt_u05(d: VDouble) -> VDouble {
    #[cfg(target_feature = "fma")]
    {
        let q: VDouble;
        let mut w: VDouble;
        let mut x: VDouble;
        let mut y: VDouble;
        let mut z: VDouble;

        // Handle negative input
        let mut d = vsel_vd_vo_vd_vd(vlt_vo_vd_vd(d, vcast_vd_d(0.0)), vcast_vd_d(f64::NAN), d);

        // Handle subnormal numbers
        let o = vlt_vo_vd_vd(d, vcast_vd_d(8.636168555094445E-78));
        d = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(d, vcast_vd_d(1.157920892373162E77)), d);
        q = vsel_vd_vo_vd_vd(o, vcast_vd_d(2.9387358770557188E-39), vcast_vd_d(1.0));

        // Initial approximation
        y = vreinterpret_vd_vm(vsub64_vm_vm_vm(
            vcast_vm_i_i(0x5fe6ec85, 0xe7de30dau32 as i32),
            vsrl64_vm_vm_i::<1>(vreinterpret_vm_vd(d)),
        ));

        // Newton-Raphson iterations with FMA
        x = vmul_vd_vd_vd(d, y);
        w = vmul_vd_vd_vd(vcast_vd_d(0.5), y);
        y = vfmanp_vd_vd_vd_vd(x, w, vcast_vd_d(0.5));

        x = vfma_vd_vd_vd_vd(x, y, x);
        w = vfma_vd_vd_vd_vd(w, y, w);
        y = vfmanp_vd_vd_vd_vd(x, w, vcast_vd_d(0.5));

        x = vfma_vd_vd_vd_vd(x, y, x);
        w = vfma_vd_vd_vd_vd(w, y, w);
        y = vfmanp_vd_vd_vd_vd(x, w, vcast_vd_d(0.5));

        x = vfma_vd_vd_vd_vd(x, y, x);
        w = vfma_vd_vd_vd_vd(w, y, w);

        // Final refinement steps
        y = vfmanp_vd_vd_vd_vd(x, w, vcast_vd_d(1.5));
        w = vadd_vd_vd_vd(w, w);
        w = vmul_vd_vd_vd(w, y);
        x = vmul_vd_vd_vd(w, d);
        y = vfmapn_vd_vd_vd_vd(w, d, x);
        z = vfmanp_vd_vd_vd_vd(w, x, vcast_vd_d(1.0));

        // Final correction
        z = vfmanp_vd_vd_vd_vd(w, y, z);
        w = vmul_vd_vd_vd(vcast_vd_d(0.5), x);
        w = vfma_vd_vd_vd_vd(w, z, y);
        w = vadd_vd_vd_vd(w, x);

        w = vmul_vd_vd_vd(w, q);

        // Handle special cases
        w = vsel_vd_vo_vd_vd(
            vor_vo_vo_vo(
                veq_vo_vd_vd(d, vcast_vd_d(0.0)),
                veq_vo_vd_vd(d, vcast_vd_d(f64::INFINITY)),
            ),
            d,
            w,
        );

        w = vsel_vd_vo_vd_vd(vlt_vo_vd_vd(d, vcast_vd_d(0.0)), vcast_vd_d(f64::NAN), w);

        w
    }

    #[cfg(not(target_feature = "fma"))]
    {
        let mut q: VDouble;
        let mut o: VMask;

        // Handle negative input
        let mut d = vsel_vd_vo_vd_vd(vlt_vo_vd_vd(d, vcast_vd_d(0.0)), vcast_vd_d(SLEEF_NAN), d);

        // Handle very small numbers
        o = vlt_vo_vd_vd(d, vcast_vd_d(8.636168555094445E-78));
        d = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(d, vcast_vd_d(1.157920892373162E77)), d);
        q = vsel_vd_vo_vd_vd(o, vcast_vd_d(2.9387358770557188E-39 * 0.5), vcast_vd_d(0.5));

        // Handle very large numbers
        o = vgt_vo_vd_vd(d, vcast_vd_d(1.3407807929942597e+154));
        d = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(d, vcast_vd_d(7.4583407312002070e-155)), d);
        q = vsel_vd_vo_vd_vd(o, vcast_vd_d(1.1579208923731620e+77 * 0.5), q);

        // Initial approximation
        let mut x = vreinterpret_vd_vm(vsub64_vm_vm_vm(
            vcast_vm_i_i(0x5fe6ec86, 0),
            vsrl64_vm_vm_i(vreinterpret_vm_vd(vadd_vd_vd_vd(d, vcast_vd_d(1e-320))), 1),
        ));

        // Newton-Raphson iterations
        x = vmul_vd_vd_vd(
            x,
            vsub_vd_vd_vd(
                vcast_vd_d(1.5),
                vmul_vd_vd_vd(vmul_vd_vd_vd(vmul_vd_vd_vd(vcast_vd_d(0.5), d), x), x),
            ),
        );
        x = vmul_vd_vd_vd(
            x,
            vsub_vd_vd_vd(
                vcast_vd_d(1.5),
                vmul_vd_vd_vd(vmul_vd_vd_vd(vmul_vd_vd_vd(vcast_vd_d(0.5), d), x), x),
            ),
        );
        x = vmul_vd_vd_vd(
            x,
            vsub_vd_vd_vd(
                vcast_vd_d(1.5),
                vmul_vd_vd_vd(vmul_vd_vd_vd(vmul_vd_vd_vd(vcast_vd_d(0.5), d), x), x),
            ),
        );
        x = vmul_vd_vd_vd(x, d);

        // Final refinement using double-double arithmetic
        let d2 = ddmul_vd2_vd2_vd2(ddadd2_vd2_vd_vd2(d, ddmul_vd2_vd_vd(x, x)), ddrec_vd2_vd(x));

        // Combine terms and apply scaling
        x = vmul_vd_vd_vd(vadd_vd_vd_vd(vd2getx_vd_vd2(d2), vd2gety_vd_vd2(d2)), q);

        // Handle special cases
        x = vsel_vd_vo_vd_vd(vispinf_vo_vd(d), vcast_vd_d(SLEEF_INFINITY), x);
        x = vsel_vd_vo_vd_vd(veq_vo_vd_vd(d, vcast_vd_d(0.0)), d, x);

        x
    }
}

#[inline(always)]
unsafe fn ddmla_vd2_vd_vd2_vd2(x: VDouble, y: VDouble2, z: VDouble2) -> VDouble2 {
    // 计算 z + (y * x)
    // 1. 先计算 y * x
    let mul = ddmul_vd2_vd2_vd(y, x);

    // 2. 再将结果与z相加
    ddadd_vd2_vd2_vd2(z, mul)
}

#[inline(always)]
unsafe fn poly2dd_b(x: VDouble, c1: VDouble2, c0: VDouble2) -> VDouble2 {
    // 计算 c1*x + c0，其中c1和c0都是双精度系数
    ddmla_vd2_vd_vd2_vd2(x, c1, c0)
}

#[inline(always)]
unsafe fn poly2dd(x: VDouble, c1: VDouble, c0: VDouble2) -> VDouble2 {
    // 将单精度系数c1转换为双精度系数
    let c1_dd = vcast_vd2_vd_vd(c1, vcast_vd_d(0.0));

    // 计算 c1*x + c0，其中c1是单精度系数，c0是双精度系数
    ddmla_vd2_vd_vd2_vd2(x, c1_dd, c0)
}

#[inline(always)]
unsafe fn poly4dd(x: VDouble, c3: VDouble, c2: VDouble2, c1: VDouble2, c0: VDouble2) -> VDouble2 {
    // 计算 x^2
    let x2 = vmul_vd_vd_vd(x, x);

    // 计算高阶项 c3*x^3 + c2*x^2
    let high = poly2dd(x, c3, c2);

    // 计算低阶项 c1*x + c0
    let low = poly2dd_b(x, c1, c0);

    // 组合: (c3*x^3 + c2*x^2)*x^2 + (c1*x + c0)
    ddmla_vd2_vd_vd2_vd2(x2, high, low)
}

#[inline(always)]
pub(crate) unsafe fn xerf_u1(a: VDouble) -> VDouble {
    let t: VDouble;
    let x = vabs_vd_vd(a);
    let mut t2: VDouble2;
    let x2 = vmul_vd_vd_vd(x, x);
    let x4 = vmul_vd_vd_vd(x2, x2);
    let x8 = vmul_vd_vd_vd(x4, x4);
    let x16 = vmul_vd_vd_vd(x8, x8);
    let o25 = vle_vo_vd_vd(x, vcast_vd_d(2.5));

    if vtestallones_i_vo64(o25) != 0 {
        // Abramowitz and Stegun approximation for x ≤ 2.5
        t = poly21d(
            x,
            x2,
            x4,
            x8,
            x16,
            -0.2083271002525222097e-14,
            0.7151909970790897009e-13,
            -0.1162238220110999364e-11,
            0.1186474230821585259e-10,
            -0.8499973178354613440e-10,
            0.4507647462598841629e-9,
            -0.1808044474288848915e-8,
            0.5435081826716212389e-8,
            -0.1143939895758628484e-7,
            0.1215442362680889243e-7,
            0.1669878756181250355e-7,
            -0.9808074602255194288e-7,
            0.1389000557865837204e-6,
            0.2945514529987331866e-6,
            -0.1842918273003998283e-5,
            0.3417987836115362136e-5,
            0.3860236356493129101e-5,
            -0.3309403072749947546e-4,
            0.1060862922597579532e-3,
            0.2323253155213076174e-3,
            0.1490149719145544729e-3,
        );

        t2 = poly4dd(
            x,
            t,
            vcast_vd2_d_d(0.0092877958392275604405, 7.9287559463961107493e-19),
            vcast_vd2_d_d(0.042275531758784692937, 1.3785226620501016138e-19),
            vcast_vd2_d_d(0.07052369794346953491, 9.5846628070792092842e-19),
        );

        t2 = ddadd_vd2_vd_vd2(vcast_vd_d(1.0), ddmul_vd2_vd2_vd(t2, x));
        t2 = ddsqu_vd2_vd2(t2);
        t2 = ddsqu_vd2_vd2(t2);
        t2 = ddsqu_vd2_vd2(t2);
        t2 = ddsqu_vd2_vd2(t2);
        t2 = ddrec_vd2_vd2(t2);
    } else {
        // Path for x > 2.5
        t = poly21d_(
            x,
            x2,
            x4,
            x8,
            x16,
            vsel_vd_vo_d_d(o25, -0.2083271002525222097e-14, -0.4024015130752621932e-18),
            vsel_vd_vo_d_d(o25, 0.7151909970790897009e-13, 0.3847193332817048172e-16),
            vsel_vd_vo_d_d(o25, -0.1162238220110999364e-11, -0.1749316241455644088e-14),
            vsel_vd_vo_d_d(o25, 0.1186474230821585259e-10, 0.5029618322872872715e-13),
            vsel_vd_vo_d_d(o25, -0.8499973178354613440e-10, -0.1025221466851463164e-11),
            vsel_vd_vo_d_d(o25, 0.4507647462598841629e-9, 0.1573695559331945583e-10),
            vsel_vd_vo_d_d(o25, -0.1808044474288848915e-8, -0.1884658558040203709e-9),
            vsel_vd_vo_d_d(o25, 0.5435081826716212389e-8, 0.1798167853032159309e-8),
            vsel_vd_vo_d_d(o25, -0.1143939895758628484e-7, -0.1380745342355033142e-7),
            vsel_vd_vo_d_d(o25, 0.1215442362680889243e-7, 0.8525705726469103499e-7),
            vsel_vd_vo_d_d(o25, 0.1669878756181250355e-7, -0.4160448058101303405e-6),
            vsel_vd_vo_d_d(o25, -0.9808074602255194288e-7, 0.1517272660008588485e-5),
            vsel_vd_vo_d_d(o25, 0.1389000557865837204e-6, -0.3341634127317201697e-5),
            vsel_vd_vo_d_d(o25, 0.2945514529987331866e-6, -0.2515023395879724513e-5),
            vsel_vd_vo_d_d(o25, -0.1842918273003998283e-5, 0.6539731269664907554e-4),
            vsel_vd_vo_d_d(o25, 0.3417987836115362136e-5, -0.3551065097428388658e-3),
            vsel_vd_vo_d_d(o25, 0.3860236356493129101e-5, 0.1210736097958368864e-2),
            vsel_vd_vo_d_d(o25, -0.3309403072749947546e-4, -0.2605566912579998680e-2),
            vsel_vd_vo_d_d(o25, 0.1060862922597579532e-3, 0.1252823202436093193e-2),
            vsel_vd_vo_d_d(o25, 0.2323253155213076174e-3, 0.1820191395263313222e-1),
            vsel_vd_vo_d_d(o25, 0.1490149719145544729e-3, -0.1021557155453465954e+0),
        );

        t2 = poly4dd(
            x,
            t,
            vsel_vd2_vo_vd2_vd2(
                o25,
                vcast_vd2_d_d(0.0092877958392275604405, 7.9287559463961107493e-19),
                vcast_vd2_d_d(-0.63691044383641748361, -2.4249477526539431839e-17),
            ),
            vsel_vd2_vo_vd2_vd2(
                o25,
                vcast_vd2_d_d(0.042275531758784692937, 1.3785226620501016138e-19),
                vcast_vd2_d_d(-1.1282926061803961737, -6.2970338860410996505e-17),
            ),
            vsel_vd2_vo_vd2_vd2(
                o25,
                vcast_vd2_d_d(0.07052369794346953491, 9.5846628070792092842e-19),
                vcast_vd2_d_d(-1.2261313785184804967e-05, -5.5329707514490107044e-22),
            ),
        );

        let mut s2 = ddadd_vd2_vd_vd2(vcast_vd_d(1.0), ddmul_vd2_vd2_vd(t2, x));
        s2 = ddsqu_vd2_vd2(s2);
        s2 = ddsqu_vd2_vd2(s2);
        s2 = ddsqu_vd2_vd2(s2);
        s2 = ddsqu_vd2_vd2(s2);
        s2 = ddrec_vd2_vd2(s2);
        t2 = vsel_vd2_vo_vd2_vd2(o25, s2, vcast_vd2_vd_vd(expk(t2), vcast_vd_d(0.0)));
    }

    t2 = ddadd2_vd2_vd2_vd(t2, vcast_vd_d(-1.0));

    let mut z = vneg_vd_vd(vadd_vd_vd_vd(vd2getx_vd_vd2(t2), vd2gety_vd_vd2(t2)));
    z = vsel_vd_vo_vd_vd(
        vlt_vo_vd_vd(x, vcast_vd_d(1e-8)),
        vmul_vd_vd_vd(x, vcast_vd_d(1.12837916709551262756245475959)),
        z,
    );
    z = vsel_vd_vo_vd_vd(vge_vo_vd_vd(x, vcast_vd_d(6.0)), vcast_vd_d(1.0), z);
    z = vsel_vd_vo_vd_vd(visinf_vo_vd(a), vcast_vd_d(1.0), z);
    z = vsel_vd_vo_vd_vd(veq_vo_vd_vd(a, vcast_vd_d(0.0)), vcast_vd_d(0.0), z);
    z = vmulsign_vd_vd_vd(z, a);

    z
}

#[inline(always)]
pub(crate) unsafe fn xhypot_u05(x: VDouble, y: VDouble) -> VDouble {
    // Take absolute values
    let x = vabs_vd_vd(x);
    let y = vabs_vd_vd(y);

    // Find min and max values
    let min = vmin_vd_vd_vd(x, y);
    let mut n = min;
    let max = vmax_vd_vd_vd(x, y);
    let mut d = max;

    // Handle subnormal numbers
    let o = vlt_vo_vd_vd(max, vcast_vd_d(SLEEF_DBL_MIN));
    n = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(n, vcast_vd_d(2_f64.powi(54))), n);
    d = vsel_vd_vo_vd_vd(o, vmul_vd_vd_vd(d, vcast_vd_d(2_f64.powi(54))), d);

    // Calculate sqrt(1 + (min/max)^2) * max using double-double arithmetic
    let t = dddiv_vd2_vd2_vd2(
        vcast_vd2_vd_vd(n, vcast_vd_d(0.0)),
        vcast_vd2_vd_vd(d, vcast_vd_d(0.0)),
    );
    let t = ddmul_vd2_vd2_vd(
        ddsqrt_vd2_vd2(ddadd2_vd2_vd2_vd(ddsqu_vd2_vd2(t), vcast_vd_d(1.0))),
        max,
    );

    // Combine high and low parts
    let mut ret = vadd_vd_vd_vd(vd2getx_vd_vd2(t), vd2gety_vd_vd2(t));

    // Handle special cases
    ret = vsel_vd_vo_vd_vd(visnan_vo_vd(ret), vcast_vd_d(f64::INFINITY), ret);
    ret = vsel_vd_vo_vd_vd(veq_vo_vd_vd(min, vcast_vd_d(0.0)), max, ret);
    ret = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(visnan_vo_vd(x), visnan_vo_vd(y)),
        vcast_vd_d(f64::NAN),
        ret,
    );
    ret = vsel_vd_vo_vd_vd(
        vor_vo_vo_vo(
            veq_vo_vd_vd(x, vcast_vd_d(f64::INFINITY)),
            veq_vo_vd_vd(y, vcast_vd_d(f64::INFINITY)),
        ),
        vcast_vd_d(f64::INFINITY),
        ret,
    );

    ret
}

#[inline(always)]
pub(crate) unsafe fn xtrunc(x: VDouble) -> VDouble {
    // 截断取整：去掉小数部分
    vtruncate2_vd_vd(x)
}

#[inline(always)]
pub(crate) unsafe fn xround(x: VDouble) -> VDouble {
    // 四舍五入取整
    vround2_vd_vd(x)
}

#[inline(always)]
pub(crate) unsafe fn xfmax(x: VDouble, y: VDouble) -> VDouble {
    vsel_vd_vo_vd_vd(
        visnan_vo_vd(y),
        x,
        vsel_vd_vo_vd_vd(vgt_vo_vd_vd(x, y), x, y),
    )
}

#[inline(always)]
pub(crate) unsafe fn xfmin(x: VDouble, y: VDouble) -> VDouble {
    vsel_vd_vo_vd_vd(
        visnan_vo_vd(y),
        x,
        vsel_vd_vo_vd_vd(vgt_vo_vd_vd(y, x), x, y),
    )
}
