//! Pictures of the app without a window or a GPU: the preview's small software rasterizer of
//! egui's meshes (`ui-preview/src/picture.rs`), fed by a kittest renderer that keeps the
//! textures the frames set. Close to what the window shows, not exact (no gamma-correct
//! blending, nearest texture sampling).
use eframe::egui::{
    self, Color32, ImageData, Pos2, Rect, TextureId, epaint::Primitive, pos2, vec2,
};
use egui_kittest::{Harness, TestRenderer};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct Texture {
    size: [usize; 2],
    pixels: Vec<Color32>,
}

struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<[f32; 4]>,
    textures: Textures,
}

impl Canvas {
    fn new(width: usize, height: usize, textures: Textures) -> Self {
        Self {
            width,
            height,
            pixels: vec![[0., 0., 0., 1.]; width * height],
            textures,
        }
    }
}

/// The textures the frames set, shared by the kittest renderer and the canvas.
type Textures = HashMap<TextureId, Texture>;

/// A kittest renderer that only keeps the textures (no GPU); the canvas draws from them.
#[derive(Clone, Default)]
pub struct Collector(Arc<Mutex<Textures>>);

impl TestRenderer for Collector {
    fn handle_delta(&mut self, delta: &mut egui::TexturesDelta) {
        let mut textures = self.0.lock().unwrap();
        update_textures(&mut textures, delta);
        for id in &delta.free {
            textures.remove(id);
        }
        // Handled: the delta must not be dropped with its changes.
        delta.clear();
    }
}

fn update_textures(textures: &mut Textures, delta: &egui::TexturesDelta) {
    for (id, changes) in &delta.set {
        for change in changes {
            let ImageData::Color(image) = &change.image;
            match change.pos {
                None => {
                    textures.insert(
                        *id,
                        Texture {
                            size: image.size,
                            pixels: image.pixels.clone(),
                        },
                    );
                }
                Some([x0, y0]) => {
                    let texture = textures.get_mut(id).expect("a texture to patch");
                    for y in 0..image.size[1] {
                        for x in 0..image.size[0] {
                            texture.pixels[(y0 + y) * texture.size[0] + x0 + x] =
                                image.pixels[y * image.size[0] + x];
                        }
                    }
                }
            }
        }
    }
}

impl Canvas {
    fn sample(&self, id: TextureId, uv: Pos2) -> [f32; 4] {
        let Some(t) = self.textures.get(&id) else {
            return [1.; 4];
        };
        let x = ((uv.x * t.size[0] as f32) as usize).min(t.size[0] - 1);
        let y = ((uv.y * t.size[1] as f32) as usize).min(t.size[1] - 1);
        let c = t.pixels[y * t.size[0] + x];
        [c.r(), c.g(), c.b(), c.a()].map(|v| v as f32 / 255.)
    }

    fn draw(&mut self, primitives: &[egui::ClippedPrimitive], scale: f32) {
        for clipped in primitives {
            let Primitive::Mesh(mesh) = &clipped.primitive else {
                continue;
            };
            let clip = Rect::from_min_max(
                (clipped.clip_rect.min.to_vec2() * scale).to_pos2(),
                (clipped.clip_rect.max.to_vec2() * scale).to_pos2(),
            )
            .intersect(Rect::from_min_size(
                Pos2::ZERO,
                vec2(self.width as f32, self.height as f32),
            ));
            for tri in mesh.indices.as_chunks::<3>().0 {
                let v = tri.map(|k| mesh.vertices[k as usize]);
                let p = v.map(|v| pos2(v.pos.x * scale, v.pos.y * scale));
                let area = edge(p[0], p[1], p[2]);
                if area.abs() < 1e-6 {
                    continue;
                }
                let bounds = Rect::from_points(&p).intersect(clip);
                if !bounds.is_positive() {
                    continue;
                }
                for y in bounds.top().floor() as usize..bounds.bottom().ceil() as usize {
                    for x in bounds.left().floor() as usize..bounds.right().ceil() as usize {
                        let c = pos2(x as f32 + 0.5, y as f32 + 0.5);
                        if !clip.contains(c) {
                            continue;
                        }
                        let w = [
                            edge(p[1], p[2], c) / area,
                            edge(p[2], p[0], c) / area,
                            edge(p[0], p[1], c) / area,
                        ];
                        if w.iter().any(|w| *w < 0.) {
                            continue;
                        }
                        let mut colour = [0f32; 4];
                        let mut uv = Pos2::ZERO;
                        for k in 0..3 {
                            let vc = v[k].color;
                            for (n, value) in [vc.r(), vc.g(), vc.b(), vc.a()].iter().enumerate() {
                                colour[n] += w[k] * *value as f32 / 255.;
                            }
                            uv += v[k].uv.to_vec2() * w[k];
                        }
                        let t = self.sample(mesh.texture_id, uv);
                        let src = [0, 1, 2, 3].map(|n| colour[n] * t[n]);
                        let dst = &mut self.pixels[y * self.width + x];
                        for n in 0..4 {
                            dst[n] = src[n] + dst[n] * (1. - src[3]);
                        }
                    }
                }
            }
        }
    }

    fn save(&self, path: &Path) {
        let file = std::fs::File::create(path).expect("the picture's file");
        let mut encoder = png::Encoder::new(
            std::io::BufWriter::new(file),
            self.width as u32,
            self.height as u32,
        );
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("a PNG header");
        let bytes: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]].map(|v| (v.clamp(0., 1.) * 255.).round() as u8))
            .collect();
        writer.write_image_data(&bytes).expect("the PNG's pixels");
    }
}

fn edge(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// The harness's last frame drawn into the PNG file *path*, at the frame's size in pixels
/// (its points times the app's zoom) times *scale*.
pub fn picture<State>(
    harness: &Harness<'_, State>,
    collector: &Collector,
    path: &Path,
    scale: f32,
) {
    let output = harness.output();
    // In pixels as the harness's window has them: the points times the zoom (the app's Size).
    let scale = scale * harness.ctx.pixels_per_point();
    let size = harness.ctx.content_rect().size();
    let (width, height) = ((size.x * scale) as usize, (size.y * scale) as usize);
    let primitives = harness
        .ctx
        .tessellate(output.shapes.clone(), output.pixels_per_point);
    let textures = collector.0.lock().unwrap().clone();
    let mut canvas = Canvas::new(width, height, textures);
    canvas.draw(&primitives, scale);
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).expect("the pictures' folder");
    }
    canvas.save(path);
}
