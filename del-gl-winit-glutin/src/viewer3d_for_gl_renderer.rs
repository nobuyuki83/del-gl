use del_gl_core::gl;

pub trait GlRenderer {
    fn initialize(&mut self, gl: &gl::Gl);
    fn draw(&mut self, gl: &gl::Gl, cam_model: &[f32; 16], cam_projection: &[f32; 16]);
}

pub struct Viewer3d {
    pub appi: crate::app_internal::AppInternal,
    pub renderer: Box<dyn GlRenderer>,
    pub view_rot: del_geo_core::view_rotation::Trackball<f32>,
    pub view_prj: del_geo_core::view_projection::Perspective<f32>,
    pub ui_state: del_gl_core::view_ui_state::UiState,
    pub is_left_btn_down_not_for_view_ctrl: bool,
    pub is_view_changed: bool,
}

impl Viewer3d {
    pub fn new(
        template: glutin::config::ConfigTemplateBuilder,
        display_builder: glutin_winit::DisplayBuilder,
        vt: Box<dyn GlRenderer>,
    ) -> Self {
        Self {
            appi: crate::app_internal::AppInternal::new(template, display_builder),
            renderer: vt,
            ui_state: del_gl_core::view_ui_state::UiState::new(),
            view_rot: del_geo_core::view_rotation::Trackball::new(),
            view_prj: del_geo_core::view_projection::Perspective {
                lens: 24.,
                near: 0.5,
                far: 3.0,
                cam_pos: [0., 0., 2.],
                proj_direction: false,
                scale: 1.,
            },
            is_left_btn_down_not_for_view_ctrl: false,
            is_view_changed: false,
        }
    }
}

impl winit::application::ApplicationHandler for Viewer3d {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(app_state) = self.appi.resumed(event_loop) else {
            return;
        };
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        use glutin::display::GetGlDisplay;
        // self.renderer.get_or_insert_with(|| {
        {
            let gl_display = &app_state.gl_context.display();
            let gl = del_gl_core::gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                use glutin::display::GlDisplay;
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            unsafe {
                gl.Enable(gl::DEPTH_TEST);
            }
            self.renderer.initialize(&gl);
        }
        assert!(self.appi.state.replace(app_state).is_none());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use glutin::prelude::GlSurface;
        self.is_left_btn_down_not_for_view_ctrl = false;
        match event {
            winit::event::WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(crate::app_internal::AppState {
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
                    // let renderer = self.renderer.as_ref().unwrap();
                    // renderer.resize(size.width as i32, size.height as i32);
                    use glutin::display::GetGlDisplay;
                    let gl_display = &gl_context.display();
                    let gl = del_gl_core::gl::Gl::load_with(|symbol| {
                        let symbol = std::ffi::CString::new(symbol).unwrap();
                        use glutin::display::GlDisplay;
                        gl_display.get_proc_address(symbol.as_c_str()).cast()
                    });
                    unsafe {
                        gl.Viewport(0, 0, size.width as i32, size.height as i32);
                    }
                }
            }
            winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::KeyboardInput {
                event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
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
        if let Some(crate::app_internal::AppState {
                        gl_context,
                        gl_surface,
                        window,
                    }) = self.appi.state.as_ref()
        {
            let img_shape = { (window.inner_size().width, window.inner_size().height) };
            let cam_model = self.view_rot.mat4_col_major();
            let cam_projection = self
                .view_prj
                .mat4_col_major(img_shape.0 as f32 / img_shape.1 as f32);
            use std::ops::DerefMut;
            let renderer = self.renderer.deref_mut();
            use glutin::display::GetGlDisplay;
            let gl_display = &gl_context.display();
            let gl = del_gl_core::gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                use glutin::display::GlDisplay;
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            unsafe {
                gl.ClearColor(0.3, 0.3, 0.3, 1.0);
                gl.Clear(gl::COLOR_BUFFER_BIT);
                gl.Clear(gl::DEPTH_BUFFER_BIT);
            }
            renderer.draw(&gl, &cam_model, &cam_projection);
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");
        self.appi.suspended();
    }
}
