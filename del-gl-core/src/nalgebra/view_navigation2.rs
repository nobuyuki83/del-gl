use nalgebra::{Matrix4, Translation3, Vector2, Vector4};

pub struct Navigation2 {
    // modelview
    translation: Vector2<f32>,
    // projection
    view_height: f32,
    pub scale: f32,
    depth_ratio: f32,
}

impl Navigation2 {
    pub fn new(win_height: f32) -> Self {
        Navigation2 {
            scale: 1.,
            view_height: win_height,
            depth_ratio: 10.,
            translation: Vector2::new(0., 0.),
        }
    }
    pub fn projection_matrix(&self, win_width: u32, win_height: u32) -> nalgebra::Matrix4<f32> {
        let asp = win_width as f32 / win_height as f32;
        let m: Matrix4<f32> = Matrix4::<f32>::new(
            1. / (self.view_height * asp),
            0.,
            0.,
            0.,
            0.,
            1. / self.view_height,
            0.,
            0.,
            0.,
            0.,
            1. / (self.view_height * self.depth_ratio),
            0.,
            0.,
            0.,
            0.,
            1.,
        );
        let d = Vector4::new(self.scale, self.scale, self.scale, 1.);
        let ms = Matrix4::from_diagonal(&d);
        ms * m
    }
    pub fn modelview_matrix(&self) -> nalgebra::Matrix4<f32> {
        let mt =
            Translation3::new(-self.translation[0], -self.translation[1], 0.0).to_homogeneous();
        mt
    }

    pub fn camera_translation(
        &mut self,
        win_width: u32,
        win_height: u32,
        cursor_dx: f64,
        cursor_dy: f64,
    ) {
        let mp = self.projection_matrix(win_width, win_height);
        let sx = (mp.get((3, 3)).unwrap() - mp.get((0, 3)).unwrap()) / mp.get((0, 0)).unwrap();
        let sy = (mp.get((3, 3)).unwrap() - mp.get((1, 3)).unwrap()) / mp.get((1, 1)).unwrap();
        self.translation[0] -= sx * cursor_dx as f32;
        self.translation[1] -= sy * cursor_dy as f32;
    }

    pub fn picking_ray(
        &self,
        win_width: u32,
        win_height: u32,
        cursor_x: f64,
        cursor_y: f64,
    ) -> (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>) {
        let mvp = self.projection_matrix(win_width, win_height) * self.modelview_matrix();
        let mvpi = mvp.try_inverse().unwrap();
        let q0 = nalgebra::Vector4::<f32>::new(cursor_x as f32, cursor_y as f32, 1., 1.);
        let q1 = nalgebra::Vector4::<f32>::new(cursor_x as f32, cursor_y as f32, -1., 1.);
        let p0 = mvpi * q0;
        let p1 = mvpi * q1;
        let dir = p1 - p0;
        (
            nalgebra::Vector3::<f32>::new(p0.x, p0.y, p0.z),
            nalgebra::Vector3::<f32>::new(dir.x, dir.y, dir.z),
        )
    }
}
