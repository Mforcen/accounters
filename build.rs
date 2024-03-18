use std::env::current_dir;
use std::process::Command;

fn main() {
    Command::new("npx")
        .args(&["tailwindcss", "-i", "base.css", "-o"])
        .arg(&format!(
            "{}/static/styles.css",
            current_dir().unwrap().into_os_string().to_str().unwrap()
        ))
        .status()
        .unwrap();
    println!("cargo:rerun-if-changed=templates/")
}
