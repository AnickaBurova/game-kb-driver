

#[derive(Debug)]
pub enum Input {
    ButtonDown (u16),
    ButtonUp (u16),
    Axis(u16, f32),
}
