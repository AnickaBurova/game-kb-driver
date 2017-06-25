

#[derive(Debug)]
pub enum Input {
    KeyDown (u16),
    KeyUp (u16),
    Axis(u16, f32),
}
