
#error -- unsupported xlC version! only xlC 13.1 and 16.1 are supported. The nvcc flag '-allow-unsupported-compiler' can be used to override this version check; however, using an unsupported host compiler may cause compilation failure or incorrect run time execution. Use at your own risk.

#endif /* __ibmxl_vrm__ && !(__ibmxl_vrm__ >= 0x0d010000 && __ibmxl_vrm__ < 0x0d020000) &&
                           !(__ibmxl_vrm__ >= 0x10010000 && __ibmxl_vrm__ < 0x10020000) */

#endif /* __powerpc__ */

#if defined(__GNUC__)

#if __GNUC__ > 15

#error -- unsupported GNU version! gcc versions later than 15 are not supported! The nvcc flag '-allow-unsupported-compiler' can be used to override this version check; however, using an unsupported host compiler may cause compilation failure or incorrect run time execution. Use at your own risk.

#endif /* __GNUC__ > 15 */


#if defined(__HORIZON__)
#if (__clang_major__ >= 22) || (__clang_major__ < 3) || ((__clang_major__ == 3) &&  (__clang_minor__ < 3))
#error -- unsupported HOS clang version! The version must be must be less than 22 and greater than 3.2 . The nvcc flag '-allow-unsupported-compiler' can be used to override this version check; however, using an unsupported host compiler may cause compilation failure or incorrect run time execution. Use at your own risk.
#endif  /* (__clang_major__ >= 22) || (__clang_major__ < 3) || ((__clang_major__ == 3) &&  (__clang_minor__ < 3)) */
