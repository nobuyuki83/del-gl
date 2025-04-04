pub mod app_internal;
pub mod viewer3d_for_image_generator;
pub mod viewer3d_for_gl_renderer;

pub fn view_navigation(
    event: winit::event::WindowEvent,
    ui_state: &mut del_gl_core::view_ui_state::UiState,
    view_prj: &mut del_geo_core::view_projection::Perspective<f32>,
    view_rot: &mut del_geo_core::view_rotation::Trackball<f32>,
) -> bool
{
    match event {
        winit::event::WindowEvent::MouseWheel {
            device_id: _,
            delta,
            ..
        } => match delta {
            winit::event::MouseScrollDelta::LineDelta(_, dy) => {
                view_prj.scale *= 1.01f32.powf(dy);
                return true;
            }
            _ => {
                return false;
            }
        },
        winit::event::WindowEvent::MouseInput {
            device_id: _,
            state,
            button,
        } => {
            if button == winit::event::MouseButton::Left
                && state == winit::event::ElementState::Pressed
            {
                ui_state.is_left_btn = true;
                if (!ui_state.is_mod_shift) && (!ui_state.is_mod_alt) {}
            }
            if button == winit::event::MouseButton::Left
                && state == winit::event::ElementState::Released
            {
                ui_state.is_left_btn = false;
            }
            return false;
        }
        winit::event::WindowEvent::ModifiersChanged(modifiers) => {
            ui_state.is_mod_alt = modifiers.state() == winit::keyboard::ModifiersState::ALT;
            ui_state.is_mod_shift = modifiers.state() == winit::keyboard::ModifiersState::SHIFT;
            return false;
        }
        winit::event::WindowEvent::CursorMoved {
            device_id: _,
            position,
            ..
        } => {
            // println!("{:?} {:?} {:?}", device_id, position, modifiers);
            ui_state.update_cursor_position(position.x, position.y);
            if ui_state.is_left_btn && ui_state.is_mod_alt {
                view_rot.camera_rotation(ui_state.cursor_dx as f32, ui_state.cursor_dy as f32);
                return true;
            }
            if ui_state.is_left_btn && ui_state.is_mod_shift {
                let asp = ui_state.win_width as f32 / ui_state.win_height as f32;
                view_prj.camera_translation(
                    asp,
                    ui_state.cursor_dx as f32,
                    ui_state.cursor_dy as f32,
                );
                return true;
            }
            return false;
        }
        _ => {
            return false;
        }
    }
}
