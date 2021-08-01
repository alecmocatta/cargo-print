use std::env::var;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	write("HOST", &var("HOST").unwrap());
}
fn write(key: &str, value: &str) {
	println!("cargo:rustc-env={}={}", key, value);
}
