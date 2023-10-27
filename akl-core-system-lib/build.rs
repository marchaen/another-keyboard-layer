fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("libakl_core_system_lib")
        .csharp_namespace("AKL.Core")
        .csharp_class_name("AklCoreNativeInterface")
        .generate_csharp_file("target/bindings/AklCoreNativeInterface.g.cs")
        .unwrap();
}
