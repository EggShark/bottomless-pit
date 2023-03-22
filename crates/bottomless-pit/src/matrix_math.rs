use crate::Vec2;

pub fn normalize_points<T: std::ops::Div<Output = T>>(point: Vec2<T>, width: T, height: T) -> Vec2<T> {
    let x = point.x / width;
    let y = point.y / height;
    Vec2{x, y}
}

pub(crate) fn get_mid_point(rectangle: wgpu_glyph::ab_glyph::Rect) -> Vec2<f32> {
    let x_mid = (rectangle.min.x + rectangle.max.x) / 2.0;
    let y_mid = (rectangle.min.y + rectangle.max.y) / 2.0;

    Vec2{ x: x_mid, y: y_mid}
}

pub fn calculate_rotation_matrix(degree: f32) -> [f32; 16] {
    let degree = degree.to_radians();
    [
        degree.cos(), -degree.sin(), 0.0, 0.0,
        degree.sin(), degree.cos(), 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

pub(crate) fn flatten_matrix(matrix: cgmath::Matrix4<f32>) -> [f32; 16] {
    [matrix.x.x, matrix.x.y, matrix.x.z, matrix.x.w, matrix.y.x, matrix.y.y, matrix.y.z, matrix.y.w, matrix.z.x, matrix.z.y, matrix.z.z, matrix.z.w, matrix.w.x, matrix.w.y, matrix.w.z, matrix.w.w]
}

pub(crate) fn unflatten_matrix(array: [f32; 16]) -> cgmath::Matrix4<f32> {
    let r1 = [array[0], array[1], array[2], array[3]];
    let r2 = [array[4], array[5], array[6], array[7]];
    let r3 = [array[8], array[9], array[10], array[11]];
    let r4 = [array[12], array[13], array[14], array[15]];
    let into = [r1, r2, r3, r4];
    cgmath::Matrix4::from(into)
}