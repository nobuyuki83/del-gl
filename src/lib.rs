pub mod view_ui_state;

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
    // include!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/gl_bindings.rs"));
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub mod array;
pub mod array_vtxcolor;

pub mod mesh;
pub mod mesh_colormap;

// folder
pub mod glutin;
pub mod nalgebra;