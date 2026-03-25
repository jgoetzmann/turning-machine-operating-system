// kernel/build.rs
// Tells cargo to re-run if linker script changes, and passes it to the linker.
// Owned by BUILD domain (Agent-01).

fn main() {
    // Re-run build script if linker script changes
    println!("cargo:rerun-if-changed=linker.ld");

    // Pass linker script to linker
    println!("cargo:rustc-link-arg=-Tlinker.ld");

    // Ensure the linker script is found relative to the kernel/ directory
    println!("cargo:rustc-link-search=.");
}
