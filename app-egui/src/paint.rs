use egui::{Color32, Pos2, Rect, Shape, Vec2};

pub fn grad(painter: &egui::Painter, rect: Rect, a: Color32, b: Color32, horizontal: bool) {
    let mut mesh = egui::Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    let (c0, c1, c2, c3) = if horizontal {
        (a, b, b, a)
    } else {
        (a, a, b, b)
    };
    for (pos, color) in [
        (rect.left_top(), c0),
        (rect.right_top(), c1),
        (rect.right_bottom(), c2),
        (rect.left_bottom(), c3),
    ] {
        mesh.vertices.push(egui::epaint::Vertex { pos, uv, color });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));
}

pub fn glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = egui::Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv,
        color,
    });
    let steps = 44;
    for i in 0..=steps {
        let angle = i as f32 / steps as f32 * std::f32::consts::TAU;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + Vec2::angled(angle) * radius,
            uv,
            color: Color32::TRANSPARENT,
        });
    }
    for i in 1..=steps {
        mesh.add_triangle(0, i as u32, i as u32 + 1);
    }
    painter.add(Shape::mesh(mesh));
}
