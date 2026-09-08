use sandlot::cpu;

fn main() {
    println!("Cache line size: {} B", cpu::cache_line_size());
    println!("# of logical CPU cores: {}", cpu::num_logical_cpu_cores());
    println!("SIMD alignment: {} B", cpu::simd_alignment());
    match cpu::system_page_size() {
        Some(ps) => println!("System page size: {ps} B"),
        None => println!("System page size cannot be determined"),
    }
    println!("System RAM: {} MiB", cpu::system_ram_mib());
    println!("Has AltiVec? {}", cpu::has_altivec());
    println!("Has ARM SIMD? {}", cpu::has_arm_simd());
    println!("Has AVX? {}", cpu::has_avx());
    println!("Has AVX2? {}", cpu::has_avx2());
    println!("Has AVX-512F? {}", cpu::has_avx512f());
    println!("Has LASX? {}", cpu::has_lasx());
    println!("Has LSX? {}", cpu::has_lsx());
    println!("Has MMX? {}", cpu::has_mmx());
    println!("Has NEON? {}", cpu::has_neon());
    println!("Has SSE? {}", cpu::has_sse());
    println!("Has SSE2? {}", cpu::has_sse2());
    println!("Has SSE3? {}", cpu::has_sse3());
    println!("Has SSE4.1? {}", cpu::has_sse4_1());
    println!("Has SSE4.2? {}", cpu::has_sse4_2());
}
