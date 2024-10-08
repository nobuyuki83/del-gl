use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let mut dest = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    dest.push("target");

    println!("{}", dest.to_str().unwrap());

    println!("cargo:rerun-if-changed=build.rs");

    println!("{:?}", std::env::var_os("OUT_DIR").unwrap());

    dest = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let mut file = File::create(dest.join("gl_bindings.rs")).unwrap();
    Registry::new(Api::Gles2, (3, 3), Profile::Core, Fallbacks::All, [])
        .write_bindings(gl_generator::StructGenerator, &mut file)
        .unwrap();
}
