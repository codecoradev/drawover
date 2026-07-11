//! Drawing utilities — Catmull-Rom spline smoothing and tiny-skia rendering.

use crate::Stroke;
use tiny_skia::{BlendMode, Paint, Path, PathBuilder, Pixmap, Stroke as SkiaStroke, Transform};

/// Catmull-Rom spline interpolation for smooth strokes.
///
/// Given a set of control points, generates a smooth curve by interpolating
/// between each pair using the Catmull-Rom formulation with a configurable
/// tension parameter.
///
/// - `tension`: 0.0 = standard Catmull-Rom, 0.5 = centripetal (sharper), 1.0 = relaxed.
///
/// Returns a dense set of interpolated points suitable for rendering a smooth path.
pub fn catmull_rom_spline(points: &[(f32, f32)], tension: f32) -> Vec<(f32, f32)> {
    if points.len() < 2 {
        return points.to_vec();
    }

    // For 2 points, just return them (can't spline a single segment)
    if points.len() == 2 {
        return points.to_vec();
    }

    let mut result: Vec<(f32, f32)> = Vec::with_capacity(points.len() * 16);
    let n = points.len();

    // Tension factor: maps [0,1] to the standard Catmull-Rom coefficient
    let s = (1.0 - tension) * 0.5;

    for i in 0..(n - 1) {
        let p0 = if i == 0 {
            // Mirror the second point around the first for the phantom start
            let dx = points[1].0 - points[0].0;
            let dy = points[1].1 - points[0].1;
            (points[0].0 - dx, points[0].1 - dy)
        } else {
            points[i - 1]
        };

        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < n {
            points[i + 2]
        } else {
            // Mirror for phantom end
            let dx = points[i + 1].0 - points[i].0;
            let dy = points[i + 1].1 - points[i].1;
            (points[i + 1].0 + dx, points[i + 1].1 + dy)
        };

        // Sample 16 subdivisions per segment
        let steps = 16;
        for step in 0..steps {
            let t = step as f32 / steps as f32;
            let t2 = t * t;
            let t3 = t2 * t;

            let x = s
                * ((2.0 * p1.0)
                    + (-p0.0 + p2.0) * t
                    + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
                    + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);

            let y = s
                * ((2.0 * p1.1)
                    + (-p0.1 + p2.1) * t
                    + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
                    + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);

            result.push((x, y));
        }
    }

    // Always include the final point
    result.push(points[n - 1]);
    result
}

/// Parse a hex color string (#RRGGBB or #RRGGBBAA) into tiny-skia Color.
fn parse_color(hex: &str) -> tiny_skia::Color {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = if hex.len() == 8 {
        (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(255),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(255),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(255),
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255),
        )
    } else if hex.len() == 6 {
        (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(255),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(255),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(255),
            255,
        )
    } else {
        // Fallback to red
        (255, 59, 48, 255)
    };

    tiny_skia::Color::from_rgba8(r, g, b, a)
}

/// Build a tiny-skia Path from a list of points using Catmull-Rom smoothing.
fn build_path(points: &[(f32, f32)]) -> Option<Path> {
    if points.is_empty() {
        return None;
    }

    let smooth = catmull_rom_spline(points, 0.0);
    let mut pb = PathBuilder::new();

    pb.move_to(smooth[0].0, smooth[0].1);

    for &(x, y) in smooth.iter().skip(1) {
        pb.line_to(x, y);
    }

    pb.finish()
}

/// Scale the alpha channel of a color by the given factor (0.0–1.0).
fn with_alpha(color: tiny_skia::Color, alpha: f32) -> tiny_skia::Color {
    let a = (color.alpha() * alpha.clamp(0.0, 1.0)) * 255.0;
    tiny_skia::Color::from_rgba8(
        (color.red() * 255.0) as u8,
        (color.green() * 255.0) as u8,
        (color.blue() * 255.0) as u8,
        a as u8,
    )
}

/// Render a single stroke onto a tiny-skia Pixmap.
///
/// Supports `tool` values: "pen", "highlighter", "eraser".
/// For the MVP, "pen" and "highlighter" render as stroked paths with different
/// opacity/blend. "eraser" is treated as a clear-blend stroke.
pub fn render_stroke(pixmap: &mut Pixmap, stroke: &Stroke) {
    if stroke.points.is_empty() {
        return;
    }

    let path = match build_path(&stroke.points) {
        Some(p) => p,
        None => return,
    };

    let color = parse_color(&stroke.color);

    match stroke.tool.as_str() {
        "eraser" => {
            // Eraser: stroke with clear blend mode
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            paint.blend_mode = BlendMode::Clear;

            let skia_stroke = SkiaStroke {
                width: stroke.width * 2.0, // Eraser is wider
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };

            pixmap.stroke_path(&path, &paint, &skia_stroke, Transform::identity(), None);
        }
        "highlighter" => {
            // Highlighter: semi-transparent, multiply blend
            let mut paint = Paint::default();
            let hl_color = with_alpha(color, stroke.opacity * 0.4);
            paint.set_color(hl_color);
            paint.anti_alias = true;
            paint.blend_mode = BlendMode::Multiply;

            let skia_stroke = SkiaStroke {
                width: stroke.width * 3.0, // Highlighter is thick
                line_cap: tiny_skia::LineCap::Butt,
                line_join: tiny_skia::LineJoin::Miter,
                ..Default::default()
            };

            pixmap.stroke_path(&path, &paint, &skia_stroke, Transform::identity(), None);
        }
        _ => {
            // Default pen tool
            let mut paint = Paint::default();
            let final_color = with_alpha(color, stroke.opacity);
            paint.set_color(final_color);
            paint.anti_alias = true;

            let skia_stroke = SkiaStroke {
                width: stroke.width,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };

            pixmap.stroke_path(&path, &paint, &skia_stroke, Transform::identity(), None);
        }
    }
}

/// Render all strokes onto a Pixmap.
pub fn render_all(pixmap: &mut Pixmap, strokes: &[Stroke]) {
    for stroke in strokes {
        render_stroke(pixmap, stroke);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_single_point() {
        let points = vec![(0.0, 0.0)];
        let result = catmull_rom_spline(&points, 0.0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_catmull_rom_two_points() {
        let points = vec![(0.0, 0.0), (10.0, 10.0)];
        let result = catmull_rom_spline(&points, 0.0);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_catmull_rom_generates_smoothed_points() {
        let points = vec![(0.0, 0.0), (10.0, 0.0), (20.0, 5.0), (30.0, 0.0)];
        let result = catmull_rom_spline(&points, 0.0);
        // 3 segments × 16 subdivisions + 1 final point = 49
        assert_eq!(result.len(), 49);
        // First point should match input
        assert!((result[0].0 - 0.0).abs() < 0.01);
        // Last point should match final input
        assert!((result[48].0 - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_color_6_digit() {
        let color = parse_color("#FF3B30");
        assert!((color.red() - 1.0).abs() < 0.01);
        assert!((color.green() - (59.0 / 255.0)).abs() < 0.01);
        assert!((color.blue() - (48.0 / 255.0)).abs() < 0.01);
        assert!((color.alpha() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_color_8_digit() {
        let color = parse_color("#FF3B3080");
        assert!((color.alpha() - (128.0 / 255.0)).abs() < 0.01);
    }

    #[test]
    fn test_parse_color_invalid_fallback() {
        let color = parse_color("not-a-color");
        assert!((color.red() - 1.0).abs() < 0.01);
        assert!((color.green() - (59.0 / 255.0)).abs() < 0.01);
        assert!((color.blue() - (48.0 / 255.0)).abs() < 0.01);
    }

    #[test]
    fn test_render_stroke_pen() {
        let mut pixmap = Pixmap::new(100, 100).unwrap();
        let stroke = Stroke {
            id: "test".to_string(),
            points: vec![(10.0, 10.0), (50.0, 50.0), (90.0, 10.0)],
            color: "#FF0000".to_string(),
            width: 5.0,
            tool: "pen".to_string(),
            opacity: 1.0,
        };
        render_stroke(&mut pixmap, &stroke);
        // Check that some pixels were drawn (not all transparent)
        let has_content = pixmap
            .pixels()
            .iter()
            .any(|p| p.red() > 0 || p.green() > 0 || p.blue() > 0);
        assert!(has_content, "Expected non-empty rendering");
    }

    #[test]
    fn test_render_stroke_empty() {
        let mut pixmap = Pixmap::new(100, 100).unwrap();
        let stroke = Stroke {
            id: "empty".to_string(),
            points: vec![],
            color: "#FF0000".to_string(),
            width: 5.0,
            tool: "pen".to_string(),
            opacity: 1.0,
        };
        render_stroke(&mut pixmap, &stroke);
        // Should be all transparent
        let all_empty = pixmap.pixels().iter().all(|p| p.alpha() == 0);
        assert!(all_empty);
    }

    #[test]
    fn test_render_all_multiple_strokes() {
        let mut pixmap = Pixmap::new(200, 200).unwrap();
        let strokes = vec![
            Stroke {
                id: "s1".to_string(),
                points: vec![(10.0, 10.0), (50.0, 50.0)],
                color: "#FF0000".to_string(),
                width: 5.0,
                tool: "pen".to_string(),
                opacity: 1.0,
            },
            Stroke {
                id: "s2".to_string(),
                points: vec![(100.0, 100.0), (150.0, 150.0)],
                color: "#00FF00".to_string(),
                width: 10.0,
                tool: "highlighter".to_string(),
                opacity: 0.5,
            },
        ];
        render_all(&mut pixmap, &strokes);
        let has_content = pixmap
            .pixels()
            .iter()
            .any(|p| p.red() > 0 || p.green() > 0 || p.blue() > 0);
        assert!(has_content);
    }

    #[test]
    fn test_with_alpha() {
        let color = tiny_skia::Color::from_rgba8(255, 0, 0, 255);
        let half = with_alpha(color, 0.5);
        assert!((half.alpha() - 0.5).abs() < 0.02);
    }
}
