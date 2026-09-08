//! CPU capability detection functions.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryCPUInfo)):
//! - [x] SDL_GetCPUCacheLineSize
//! - [x] SDL_GetNumLogicalCPUCores
//! - [x] SDL_GetSIMDAlignment
//! - [x] SDL_GetSystemPageSize
//! - [x] SDL_GetSystemRAM
//! - [x] SDL_HasAltiVec
//! - [x] SDL_HasARMSIMD
//! - [x] SDL_HasAVX
//! - [x] SDL_HasAVX2
//! - [x] SDL_HasAVX512F
//! - [x] SDL_HasLASX
//! - [x] SDL_HasLSX
//! - [x] SDL_HasMMX
//! - [x] SDL_HasNEON
//! - [x] SDL_HasSSE
//! - [x] SDL_HasSSE2
//! - [x] SDL_HasSSE3
//! - [x] SDL_HasSSE41
//! - [x] SDL_HasSSE42
//! - [ ] SDL_HasSVE2 (will be available in SDL 3.6.0)

use std::num::NonZero;

use sdl3_sys::cpuinfo::*;

/// Determine the L1 cache line size of the CPU, in bytes.
///
/// # Remarks
///
/// This is useful for determining multi-threaded structure padding or SIMD
/// prefetch sizes.
#[doc(alias = "SDL_GetCPUCacheLineSize")]
pub fn cache_line_size() -> i32 {
    unsafe { SDL_GetCPUCacheLineSize() }
}

/// Get the number of logical CPU cores available.
///
/// On CPUs that include technologies such as hyperthreading, the number of
/// logical cores may be more than the number of physical cores.
#[doc(alias = "SDL_GetNumLogicalCPUCores")]
pub fn num_logical_cpu_cores() -> i32 {
    unsafe { SDL_GetNumLogicalCPUCores() }
}

/// Report the alignment this system needs for SIMD allocations, in bytes.
///
/// # Remarks
///
/// This will return the minimum number of bytes to which a pointer must be
/// aligned to be compatible with SIMD instructions on the current machine.
/// For example, if the machine supports SSE only, it will return 16, but if
/// it supports AVX-512F, it'll return 64 (etc). This only reports values
/// for instruction sets SDL knows about, so if your SDL build doesn't have
/// [`has_avx512f`], then it might return 16 for the SSE support it sees and
/// not 64 for the AVX-512 instructions that exist but SDL doesn't know
/// about. Plan accordingly.
#[doc(alias = "SDL_GetSIMDAlignment")]
pub fn simd_alignment() -> usize {
    unsafe { SDL_GetSIMDAlignment() }
}

/// Report the size of a page of memory, in bytes.
///
/// Returns [`None`] if SDL can't determine this information (it does not
/// set an error string in this case; defaulting to 4096 is often a
/// reasonable option).
///
/// # Remarks
///
/// Different platforms might have different memory page sizes. In current
/// times, 4 kilobytes is not unusual, but newer systems are moving to
/// larger page sizes, and esoteric platforms might have any unexpected
/// size.
#[doc(alias = "SDL_GetSystemPageSize")]
pub fn system_page_size() -> Option<NonZero<i32>> {
    NonZero::new(unsafe { SDL_GetSystemPageSize() })
}

/// Get the amount of RAM configured in the system, in MiB.
#[doc(alias = "SDL_GetSystemRAM")]
pub fn system_ram_mib() -> i32 {
    unsafe { SDL_GetSystemRAM() }
}

/// Determine whether the CPU has AltiVec features.
///
/// This always returns false on CPUs that aren't using PowerPC instruction
/// sets.
#[doc(alias = "SDL_HasAltiVec")]
pub fn has_altivec() -> bool {
    unsafe { SDL_HasAltiVec() }
}

/// Determine whether the CPU has ARM SIMD (ARMv6) features.
///
/// This is different from ARM NEON, which is a different instruction set.
/// This always returns false on CPUs that aren't using ARM instruction
/// sets.
#[doc(alias = "SDL_HasARMSIMD")]
pub fn has_arm_simd() -> bool {
    unsafe { SDL_HasARMSIMD() }
}

/// Determine whether the CPU has AVX features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasAVX")]
pub fn has_avx() -> bool {
    unsafe { SDL_HasAVX() }
}

/// Determine whether the CPU has AVX2 features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasAVX2")]
pub fn has_avx2() -> bool {
    unsafe { SDL_HasAVX2() }
}

/// Determine whether the CPU has AVX-512F (foundation) features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasAVX512F")]
pub fn has_avx512f() -> bool {
    unsafe { SDL_HasAVX512F() }
}

/// Determine whether the CPU has LASX (LOONGARCH SIMD) features.
///
/// This always returns false on CPUs that aren't using LOONGARCH
/// instruction sets.
#[doc(alias = "SDL_HasLASX")]
pub fn has_lasx() -> bool {
    unsafe { SDL_HasLASX() }
}

/// Determine whether the CPU has LSX (LOONGARCH SIMD) features.
///
/// This always returns false on CPUs that aren't using LOONGARCH
/// instruction sets.
#[doc(alias = "SDL_HasLSX")]
pub fn has_lsx() -> bool {
    unsafe { SDL_HasLSX() }
}

/// Determine whether the CPU has MMX features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasMMX")]
pub fn has_mmx() -> bool {
    unsafe { SDL_HasMMX() }
}

/// Determine whether the CPU has NEON (ARM SIMD) features.
///
/// This always returns false on CPUs that aren't using ARM instruction
/// sets.
#[doc(alias = "SDL_HasNEON")]
pub fn has_neon() -> bool {
    unsafe { SDL_HasNEON() }
}

/// Determine whether the CPU has SSE features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasSSE")]
pub fn has_sse() -> bool {
    unsafe { SDL_HasSSE() }
}

/// Determine whether the CPU has SSE2 features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasSSE2")]
pub fn has_sse2() -> bool {
    unsafe { SDL_HasSSE2() }
}

/// Determine whether the CPU has SSE3 features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasSSE3")]
pub fn has_sse3() -> bool {
    unsafe { SDL_HasSSE3() }
}

/// Determine whether the CPU has SSE4.1 features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasSSE41")]
pub fn has_sse4_1() -> bool {
    unsafe { SDL_HasSSE41() }
}

/// Determine whether the CPU has SSE4.2 features.
///
/// This always returns false on CPUs that aren't using Intel instruction
/// sets.
#[doc(alias = "SDL_HasSSE42")]
pub fn has_sse4_2() -> bool {
    unsafe { SDL_HasSSE42() }
}
