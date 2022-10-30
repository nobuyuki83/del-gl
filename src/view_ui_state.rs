
pub struct UiState {
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub cursor_dx: f64,
    pub cursor_dy: f64,
    pub win_height: u32,
    pub win_width: u32,
    pub is_left_btn: bool,
    pub is_mod_alt: bool,
    pub is_mod_shift: bool
}


impl UiState {
    pub fn new() -> Self {
        UiState {
            cursor_x: 0.,
            cursor_y: 0.,
            cursor_dx: 0.,
            cursor_dy: 0.,
            win_height: 300,
            win_width: 300,
            is_left_btn: false,
            is_mod_alt: false,
            is_mod_shift: false,
        }
    }

    pub fn update_cursor_position(&mut self, x: f64, y: f64) {
        let fw = self.win_width as f64;
        let fh = self.win_height as f64;
        let x0 = self.cursor_x;
        let y0 = self.cursor_y;
        self.cursor_x = (2.0 * x - fw) / fw;
        self.cursor_y = (fh - 2.0 * y) / fh;
        self.cursor_dx = self.cursor_x - x0;
        self.cursor_dy = self.cursor_y - y0;
    }
}