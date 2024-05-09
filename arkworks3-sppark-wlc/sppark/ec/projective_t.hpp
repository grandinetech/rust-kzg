// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#ifndef __PROJECTIVE_T_HPP__
#define __PROJECTIVE_T_HPP__

#ifndef __CUDA_ARCH__
# undef  __host__
# define __host__
# undef  __device__
# define __device__
# undef  __noinline__
# define __noinline__
#endif

template<class field_t> class projective_t {
    field_t X, Y, Z;

public:
#ifdef __NVCC__
    class affine_inf_t { friend projective_t;
        field_t X, Y;
        int inf[sizeof(field_t)%16 ? 2 : 4];

        inline __device__ bool is_inf() const
        {   return inf[0]&1 != 0;   }
    };
#else
    class affine_inf_t { friend projective_t;
        field_t X, Y;
        bool inf;

        inline __device__ bool is_inf() const
        {   return inf;   }
    };
#endif

    class affine_t { friend projective_t;
        field_t X, Y;

        inline __device__ bool is_inf() const
        {   return (bool)(X.is_zero() & Y.is_zero());   }

    public:
        inline affine_t& operator=(const projective_t& a)
        {
            Y = a.Z.inv();
            X = Y * a.X;    // X/Z
            Y *= a.Y;       // Y/Z
            return *this;
        }
        inline affine_t(const projective_t& a)  { *this = a; }
    };

    inline operator affine_t() const      { return affine_t(*this); }

    template<class affine_t>
    inline __device__ projective_t& operator=(const affine_t& a)
    {
        X = a.X;
        Y = a.Y;
        // this works as long as |a| was confirmed to be non-infinity
        Z = field_t::one();
        return *this;
    }

    template<class affine_t>
    inline __device__ void projective_to_affine(affine_t& a) {
        a.Y = Z.inv();
        a.X = a.Y * X;
        a.Y *= Y;
    }

    inline __device__ void projective_print() {
        field_t x;
        field_t y;
        y = Z.inv();
        x = y * X;
        y *= Y;

        x.from();
        y.from();
        printf("--------\n");
        for (int i = 11; i >= 0; i--) {
            printf("%08x", *((uint32_t*)(&x) + i));
        }
        printf("\n");
        for (int i = 11; i >= 0; i--) {
            printf("%08x", *((uint32_t*)(&y) + i));
        }
        printf("\n--------\n");
    }


    inline __device__ operator jacobian_t<field_t>() const
    {   return jacobian_t<field_t>{ X*Z, (Y*Z)*Z, Z };   }

    inline __device__ bool is_inf() const { return (bool)(Z.is_zero()); }
    inline __device__ void inf()          { Z.zero(); }

    inline __device__ void neg(bool subtract = false) {
        this->Y.cneg(subtract);
    }

    /*
     * https://hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#addition-add-2015-rcb
     * https://hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#addition-madd-2015-rcb
     * https://hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#addition-mmadd-1998-cmo
     */
    __device__ void add(const projective_t& p2)
    {
        if (p2.is_inf()) {
            return;
        } else if (is_inf()) {
            *this = p2;
            return;
        }

#ifdef __CUDA_ARCH__
        projective_t p31 = *this;
#else
        projective_t& p31 = *this;
#endif

        if (p31.Z.is_one()) {
            if (p2.Z.is_one()) {                /* Z1==1 && Z2==1 */
                field_t X0, Y0, uu, vv, R, A;

                X0 = p2.X - p31.X;
                Y0 = p2.Y - p31.Y;
                uu = Y0^2;
                vv = X0^2;
                p31.Z = X0 * vv;
                R = vv * p31.X;
                A = uu - p31.Z;
                A -= R;
                A -= R;
                p31.X = X0 * A;
                p31.Y *= p31.Z;
                p31.Y = Y0 * (R - A) - p31.Y;
            } else {                            /* Z1==1 && Z2!=1 */
                field_t t0, t1, t3, t4, t5;

                t0 = p2.X * p31.X;
                t1 = p2.Y * p31.Y;
                t3 = p31.X + p31.Y;
                t4 = p2.X + p2.Y;
                t3 *= t4;
                t4 = t0 + t1;
                t3 -= t4;
                t4 = p31.X * p2.Z;
                t4 += p2.X;
                t5 = p31.Y * p2.Z;
                t5 += p2.Y;
                p31.X = p2.Z + p2.Z;
                p31.X += p2.Z;
                p31.Z = t1 + p31.X;
                p31.X = t1 - p31.X;
                p31.Y = p31.X * p31.Z;
                t1 = t0 + t0;
                t1 += t0;
                t4 += (t4 + t4);
                t0 = t1 * t4;
                p31.Y += t0;
                t0 = t5 * t4;
                p31.X *= t3;
                p31.X -= t0;
                t0 = t3 * t1;
                p31.Z *= t5;
                p31.Z += t0;
            }
        } else if (p2.Z.is_one()) {             /* Z1!=1 && Z2==1 */
            field_t t0, t1, t3, t4, t5;

            t0 = p31.X * p2.X;
            t1 = p31.Y * p2.Y;
            t3 = p2.X + p2.Y;
            t4 = p31.X + p31.Y;
            t3 *= t4;
            t4 = t0 + t1;
            t3 -= t4;
            t4 = p2.X * p31.Z;
            t4 += p31.X;
            t5 = p2.Y * p31.Z;
            t5 += p31.Y;
            p31.X = p31.Z + p31.Z;
            p31.X += p31.Z;
            p31.Z = t1 + p31.X;
            p31.X = t1 - p31.X;
            p31.Y = p31.X * p31.Z;
            t1 = t0 + t0;
            t1 += t0;
            t4 += (t4 + t4);
            t0 = t1 * t4;
            p31.Y += t0;
            t0 = t5 * t4;
            p31.X *= t3;
            p31.X -= t0;
            t0 = t3 * t1;
            p31.Z *= t5;
            p31.Z += t0;
        } else {                                /* Z1!=1 && Z2!=1 */
            field_t t0, t1, t2, t3, t4, t5;

            t0 = p31.X * p2.X;
            t1 = p31.Y * p2.Y;
            t2 = p31.Z * p2.Z;
            t3 = p31.X + p31.Y;
            t4 = p2.X + p2.Y;
            t3 *= t4;
            t4 = t0 + t1;
            t3 -= t4;
            t4 = p31.X + p31.Z;
            t5 = p2.X + p2.Z;
            t4 *= t5;
            t5 = t0 + t2;
            t4 -= t5;
            t5 = p31.Y + p31.Z;
            p31.X = p2.Y + p2.Z;
            t5 *= p31.X;
            p31.X = t1 + t2;
            t5 -= p31.X;
            p31.X = t2 + t2;
            p31.X += t2;
            p31.Z = t1 + p31.X;
            p31.X = t1 - p31.X;
            p31.Y = p31.X * p31.Z;
            t1 = t0 + t0;
            t1 += t0;
            t4 += (t4 + t4);
            t0 = t1 * t4;
            p31.Y += t0;
            t0 = t5 * t4;
            p31.X *= t3;
            p31.X -= t0;
            t0 = t3 * t1;
            p31.Z *= t5;
            p31.Z += t0;
        }
#ifdef __CUDA_ARCH__
        *this = p31;
#endif
    }

    /*
     * https://hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#doubling-dbl-2015-rcb
     * https://hyperelliptic.org/EFD/g1p/auto-shortw-projective.html#doubling-mdbl-2007-bl
     */

    __device__ void dbl()
    {
        if (is_inf()) {
            return;
        }
#ifdef __CUDA_ARCH__
        projective_t p31 = *this;
#else
        projective_t& p31 = *this;
#endif
        if (p31.Z.is_one()) {
            field_t XX, w, Y1Y1, R, RR, B, h;

            XX = p31.X^2;
            w = XX + XX;
            w += XX;
            Y1Y1 = p31.Y^2;
            R = Y1Y1 + Y1Y1;
            p31.Z = p31.Y * R;
            p31.Z += p31.Z;
            p31.Z += p31.Z;
            RR = R^2;
            B = p31.X + R;
            B *= B;
            B -= XX;
            B -= RR;
            h = w^2;
            h -= B;
            h -= B;
            p31.X = h * p31.Y;
            p31.X += p31.X;
            p31.Y = B - h;
            p31.Y *= w;
            p31.Y -= RR;
            p31.Y -= RR;

        } else {
            field_t t0, t1, t2, t3;
            field_t Y1, Z1;

            Y1 = p31.Y;
            Z1 = p31.Z;

            t0 = p31.X^2;
            t1 = p31.Y^2;
            t2 = p31.Z^2;
            t3 = p31.X * p31.Y;
            t3 += t3;
            p31.Z *= p31.X;
            p31.Z += p31.Z;
            p31.Y = t2 + t2;
            p31.Y += t2;
            p31.X = t1 - p31.Y;
            p31.Y += t1;
            p31.Y *= p31.X;
            p31.X *= t3;
            p31.Z += (p31.Z + p31.Z);
            t3 = p31.Z;
            p31.Z = t0 + t0;
            t0 += p31.Z;
            t0 *= t3;
            p31.Y += t0;
            t2 = Y1 * Z1;
            t2 += t2;
            t0 = t2 * t3;
            p31.X -= t0;
            p31.Z = t2 * t1;
            p31.Z += p31.Z;
            p31.Z += p31.Z;
        }

#ifdef __CUDA_ARCH__
        *this = p31;
#endif
    }

};
#endif
