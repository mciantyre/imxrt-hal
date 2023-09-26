fn main() {
    cxx_build::bridge("src/lib.rs")
        .no_default_flags(true)
        .file("src/pit.cpp")
        .file("src/pit_ctor.cpp")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        .flag("-mcpu=cortex-m7")
        .flag("-mfloat-abi=hard")
        .flag("-mfpu=fpv5-d16")
        .std("c++14")
        .compile("cxx-pit");

    println!("cargo:rerun-if-changed=src/pit.cpp");
    println!("cargo:rerun-if-changed=src/pit_ctor.cpp");
    println!("cargo:rerun-if-changed=include/pit.hpp");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
}
