fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/ffi.rs")
        .csharp_dll_name("akl_core_system_lib")
        .csharp_namespace("AKL.Core")
        .csharp_class_name("AklCoreNativeInterface")
        .generate_csharp_file("target/bindings/AklCoreNativeInterface.g.cs")
        .unwrap();
}
