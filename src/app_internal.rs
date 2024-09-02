use std::error::Error;
use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::ops::Deref;

use crate::gl::types::GLfloat;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};

use glutin_winit::{DisplayBuilder, GlWindow};

// Find the config with the maximum number of samples, so our triangle will be
// smooth.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

pub struct AppState {
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    pub window: Window,
}

pub struct AppInternal {
    template: ConfigTemplateBuilder,
    display_builder: DisplayBuilder,
    pub exit_state: Result<(), Box<dyn Error>>,
    not_current_gl_context: Option<NotCurrentContext>,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    pub state: Option<AppState>,
}

impl AppInternal {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        Self {
            template,
            display_builder,
            exit_state: Ok(()),
            not_current_gl_context: None,
            state: None,
        }
    }

    pub fn resumed(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Option<(AppState)> {
        let (mut window, gl_config) = match self.display_builder.clone().build(
            event_loop,
            self.template.clone(),
            gl_config_picker,
        ) {
            Ok(ok) => ok,
            Err(e) => {
                self.exit_state = Err(e);
                event_loop.exit();
                return None;
            }
        };

        println!("Picked a config with {} samples", gl_config.num_samples());

        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());

        // XXX The display could be obtained from any object created by it, so we can
        // query it from the config.
        let gl_display = gl_config.display();

        // The context creation part.
        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);

        // There are also some old devices that support neither modern OpenGL nor GLES.
        // To support these we can try and create a 2.1 context.
        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(raw_window_handle);

        // Reuse the uncurrented context from a suspended() call if it exists, otherwise
        // this is the first time resumed() is called, where the context still
        // has to be created.
        let not_current_gl_context = self
            .not_current_gl_context
            .take()
            .unwrap_or_else(|| unsafe {
                gl_display
                    .create_context(&gl_config, &context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &fallback_context_attributes)
                            .unwrap_or_else(|_| {
                                gl_display
                                    .create_context(&gl_config, &legacy_context_attributes)
                                    .expect("failed to create context")
                            })
                    })
            });

        #[cfg(android_platform)]
        println!("Android window available");

        let window = window.take().unwrap_or_else(|| {
            let window_attributes = Window::default_attributes()
                .with_transparent(true)
                .with_title("Glutin triangle gradient example (press Escape to exit)");
            glutin_winit::finalize_window(event_loop, window_attributes, &gl_config).unwrap()
        });

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        // Make it current.
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        // Try setting vsync.
        if let Err(res) = gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            eprintln!("Error setting vsync: {res:?}");
        }
        Some(
            (AppState {
                gl_context,
                gl_surface,
                window,
            }),
        )
    }

    pub fn suspended(&mut self) {
        // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        // the window back to the system.
        let gl_context = self.state.take().unwrap().gl_context;
        assert!(self
            .not_current_gl_context
            .replace(gl_context.make_not_current().unwrap())
            .is_none());
    }
}

// ---------------
