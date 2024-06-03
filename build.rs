use std::env::current_dir;
use std::process::Command;

fn main() {
    Command::new("npx")
        .args(&["tailwindcss", "-i", "base.css", "-o"])
        .arg(&format!(
            "{}/webserver/src/static/styles.css",
            current_dir().unwrap().into_os_string().to_str().unwrap()
        ))
        .status()
        .unwrap();
    println!("cargo:rerun-if-changed=templates/");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_arch == "x86_64" && target_os == "android" {
        println!("cargo:rustc-link-search=/home/forcen/Android/Sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/17/lib/linux/");
        println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
    }
}
