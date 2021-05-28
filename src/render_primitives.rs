pub enum RenderPrimitives {
    Clear {
        color: (u32, f32),
    },
    VertCenteredText {
        text: String,
        rect: (f32, f32, f32, f32), // x y w h
        color_fill: (u32, f32),
        color_stroke: (u32, f32),
        weight_stroke: f32,
        size_font: f32,
        size_space: f32,
    },
}
