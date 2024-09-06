use nalgebra::{Matrix4, Translation3, UnitQuaternion, Vector3, Vector4};

pub struct Navigation3 {
    // modelview
    translation: [f32; 3],
    quaternion: [f32; 4],
    // projection
    view_height: f32,
    pub scale: f32,
    depth_ratio: f32,
}

impl Navigation3 {
    pub fn new(win_height: f32) -> Self {
        Navigation3 {
            scale: 1.,
            view_height: win_height,
            depth_ratio: 10.,
            //
            translation: [0., 0., 0.],
            quaternion: [0., 0., 0., 1.],
        }
    }
    pub fn projection_matrix(&self, win_width: u32, win_height: u32) -> [f32; 16] {
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
        let a = ms * m;
        [
            a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11], a[12], a[13],
            a[14], a[15],
        ]
    }
    pub fn modelview_matrix(&self) -> [f32; 16] {
        let mr = del_geo_core::quat::to_mat4_col_major(&self.quaternion);
        let mt = del_geo_core::mat4_col_major::translate(&[
            -self.translation[0],
            -self.translation[1],
            -self.translation[2],
        ]);
        del_geo_core::mat4_col_major::multmat(&mt, &mr)
    }
    pub fn camera_rotation(&mut self, cursor_dx: f64, cursor_dy: f64) {
        let dx = cursor_dx as f32;
        let dy = cursor_dy as f32;
        let a: f32 = (dx * dx + dy * dy).sqrt();
        if a == 0.0 {
            return;
        }
        let dq =
            del_geo_core::quat::normalized(&del_geo_core::quat::from_axisangle(&[-dy, dx, 0.]));
        self.quaternion = del_geo_core::quat::mult_quaternion(&dq, &self.quaternion);
        // println!("{:?}",self.quaternion);
    }
    pub fn camera_translation(
        &mut self,
        win_width: u32,
        win_height: u32,
        cursor_dx: f64,
        cursor_dy: f64,
    ) {
        let mp = self.projection_matrix(win_width, win_height);
        let sx = (mp[3 + 4 * 3] - mp[0 + 4 * 3]) / mp[0 + 4 * 0];
        let sy = (mp[3 + 4 * 3] - mp[1 + 4 * 3]) / mp[1 + 4 * 1];
        self.translation[0] -= sx * cursor_dx as f32;
        self.translation[1] -= sy * cursor_dy as f32;
    }

    /*
    pub fn picking_ray(
        &self,
        win_width: u32,
        win_height: u32,
        cursor_x: f64,
        cursor_y: f64) -> (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>) {
        let mvp = del_geo_core::mat4_col_major::multmat(
            &self.projection_matrix(win_width, win_height),
            &self.modelview_matrix());
        let mvpi = del_geo_core::mat4_col_major::try_inverse(&mvp).unwrap();
        let q0 = nalgebra::Vector4::<f32>::new(cursor_x as f32, cursor_y as f32, 1., 1.);
        let q1 = nalgebra::Vector4::<f32>::new(cursor_x as f32, cursor_y as f32, -1., 1.);
        let p0 = mvpi * q0;
        let p1 = mvpi * q1;
        let dir = p1 - p0;
        (
            nalgebra::Vector3::<f32>::new(p0.x, p0.y, p0.z),
            nalgebra::Vector3::<f32>::new(dir.x, dir.y, dir.z)
        )
    }
     */
}
