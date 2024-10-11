use glutin::display::GlDisplay;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};
//
use del_gl_core::gl;
use crate::app_internal;

pub trait Content {
    fn new() -> Self;
    fn compute_image(&mut self,
                     img_shape: (usize, usize),
                     cam_projection: &[f32;16],
                     cam_model: &[f32;16]) -> Vec<u8>;
}

pub struct MyApp<C: Content>{
    pub content: C,
    pub appi: crate::app_internal::AppInternal,
    pub renderer: Option<del_gl_core::drawer_array_xyzuv::Drawer>,
    pub view_rot: del_geo_core::view_rotation::Trackball,
    pub view_prj: del_geo_core::view_projection::Perspective,
    pub ui_state: del_gl_core::view_ui_state::UiState,
}

impl<C: Content> MyApp<C> {
    pub fn new(
        template: glutin::config::ConfigTemplateBuilder,
        display_builder: glutin_winit::DisplayBuilder,
    ) -> Self {
        //
        Self {
            appi: app_internal::AppInternal::new(template, display_builder),
            renderer: None,
            ui_state: del_gl_core::view_ui_state::UiState::new(),
            view_rot: del_geo_core::view_rotation::Trackball::new(),
            view_prj: del_geo_core::view_projection::Perspective {
                lens: 24.,
                near: 0.5,
                far: 3.0,
                cam_pos: [0., 0., 2.],
                proj_direction: true,
                scale: 1.,
            },
            content: C::new()
        }
    }
}

impl<C: Content> ApplicationHandler for MyApp<C> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        use glutin::display::GetGlDisplay;
        let Some(app_state) = self.appi.resumed(event_loop) else {
            return;
        };
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        self.renderer.get_or_insert_with(|| {
            let gl_display = &app_state.gl_context.display();
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            let mut render = del_gl_core::drawer_array_xyzuv::Drawer::new(gl);
            render.init_gl();
            render
        });
        unsafe {
            //
            let Some(rndr) = &self.renderer else {
                panic!();
            };
            let gl = &rndr.gl;
            {
                #[rustfmt::skip]
                static VERTEX_DATA: [f32; 24] = [
                    -1.0, -1.0, 0., 1.,
                    1.0, -1.0, 1., 1.,
                    1.0, 1.0, 1., 0.,
                    //
                    -1.0, -1.0, 0., 1.,
                    1.0, 1.0, 1., 0.,
                    -1.0, 1.0, 0., 0.
                ];
                gl.BindBuffer(gl::ARRAY_BUFFER, rndr.vbo);
                gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    VERTEX_DATA.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
            }
        }
        assert!(self.appi.state.replace(app_state).is_none());
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");
        self.appi.suspended();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        use glutin::prelude::GlSurface;
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(app_internal::AppState {
                                gl_context,
                                gl_surface,
                                window: _,
                            }) = self.appi.state.as_ref()
                {
                    gl_surface.resize(
                        gl_context,
                        std::num::NonZeroU32::new(size.width).unwrap(),
                        std::num::NonZeroU32::new(size.height).unwrap(),
                    );
                    let renderer = self.renderer.as_ref().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                    self.ui_state.win_width = size.width;
                    self.ui_state.win_height = size.height;
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    logical_key: Key::Named(NamedKey::Escape),
                    ..
                },
                ..
            } => event_loop.exit(),
            _ => (),
        }
        let redraw = crate::view_navigation(
            event,
            &mut self.ui_state,
            &mut self.view_prj,
            &mut self.view_rot,
        );
        if redraw {
            if let Some(state) = &self.appi.state {
                state.window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        use glutin::prelude::GlSurface;
        if let Some(app_internal::AppState {
                        gl_context,
                        gl_surface,
                        window,
                    }) = self.appi.state.as_ref()
        {
            let img_shape = {
                (
                    window.inner_size().width as usize,
                    window.inner_size().height as usize,
                )
            };
            let cam_model = self.view_rot.mat4_col_major();
            let cam_projection = self
                .view_prj
                .mat4_col_major(img_shape.0 as f32 / img_shape.1 as f32);
            let img_data = self.content.compute_image(
                img_shape, &cam_projection, &cam_model);
            assert_eq!(img_data.len(), img_shape.0 * img_shape.1 * 3);
            //println!("{:?}",img.color());
            let Some(ref rndr) = self.renderer else {
                panic!();
            };
            let gl = &rndr.gl;
            unsafe {
                gl.BindTexture(gl::TEXTURE_2D, rndr.id_tex);
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB.try_into().unwrap(),
                    img_shape.0.try_into().unwrap(),
                    img_shape.1.try_into().unwrap(),
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img_data.as_ptr() as *const _,
                );
                gl.GenerateMipmap(gl::TEXTURE_2D);
            }
            //
            let renderer = self.renderer.as_ref().unwrap();
            renderer.draw();
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

