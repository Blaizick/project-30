use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=../binding-lib");
    println!("cargo:rustc-env=CC_ENABLE_DEBUG_OUTPUT=1");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .unwrap();

    let binding_lib_dir = manifest_dir.parent().unwrap().join("binding-lib");

    let status = Command::new("dotnet")
        .args(["build", "--configuration", "Release"])
        .current_dir(&binding_lib_dir)
        .status()
        .expect("failed to execute `dotnet build`");

    if !status.success() {
        panic!("dotnet build failed with status: {status}");
    }

    let binding_lib_target_dir = binding_lib_dir.join("bin").join("Release").join("net10.0");
    let runtime_config_name = "binding-lib.runtimeconfig.json";
    let dll_name = "binding-lib.dll";

    let target_bindings_dir = target_dir.join("bindings");
    
    fs::create_dir_all(&target_bindings_dir).unwrap();

    fs::copy(binding_lib_target_dir.join(runtime_config_name), target_bindings_dir.join(runtime_config_name)).unwrap();
    fs::copy(binding_lib_target_dir.join(dll_name), target_bindings_dir.join(dll_name)).unwrap();

    let src = PathBuf::from("assets");
    let dst = target_dir.join("assets");

    copy_dir_all(&src, &dst).unwrap();
}

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}