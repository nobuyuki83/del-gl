pub mod gl {
    #![allow(
    clippy::manual_non_exhaustive,
    clippy::too_many_arguments,
    clippy::unused_unit,
    clippy::upper_case_acronyms,
    non_camel_case_types
    )]

    pub use self::Gles2 as Gl;

    // gl_bindings.rs is generated in build.rs using https://crates.io/crates/gl_generator
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/build/gl_bindings.rs"));
}

pub mod drawer_arrayposcolor;
pub mod drawer_meshpos;
pub mod drawer_meshposcolor;
pub mod glutin_viewer2;
pub mod glutin_viewer3;