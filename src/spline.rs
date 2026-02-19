use eframe::egui;

/// Monotone Cubic Spline interpolation (Fritsch-Carlson)
/// This provides a very smooth curve that doesn't "overshoot" like standard cubic splines.
pub fn interpolate_spline(points: &[egui::Pos2]) -> [u8; 256] {
    let mut lut = [0u8; 256];
    if points.is_empty() {
        for i in 0..256 { lut[i] = i as u8; }
        return lut;
    }

    let mut pts = points.to_vec();
    pts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

    // 1. Calculate tangents (slopes)
    let n = pts.len();
    if n < 2 {
        for i in 0..256 { lut[i] = (pts[0].y.clamp(0.0, 1.0) * 255.0) as u8; }
        return lut;
    }

    let mut dx = vec![0.0; n - 1];
    let mut dy = vec![0.0; n - 1];
    let mut slope = vec![0.0; n - 1];
    for i in 0..n - 1 {
        dx[i] = pts[i + 1].x - pts[i].x;
        dy[i] = pts[i + 1].y - pts[i].y;
        slope[i] = dy[i] / dx[i];
    }

    // 2. Initialize tangents at points
    let mut m = vec![0.0; n];
    m[0] = slope[0];
    for i in 1..n - 1 {
        m[i] = (slope[i - 1] + slope[i]) / 2.0;
    }
    m[n - 1] = slope[n - 2];

    // 3. Force monotonicity
    for i in 0..n - 1 {
        if slope[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let a = m[i] / slope[i];
            let b = m[i + 1] / slope[i];
            let h = (a * a + b * b).sqrt();
            if h > 3.0 {
                let t = 3.0 / h;
                m[i] = t * a * slope[i];
                m[i + 1] = t * b * slope[i];
            }
        }
    }

    // 4. Interpolate
    for x_idx in 0..256 {
        let x = x_idx as f32 / 255.0;
        
        let y = if x <= pts[0].x {
            // Linear ramp from (0,0) to the first point
            if pts[0].x > 0.0 {
                (x / pts[0].x) * pts[0].y
            } else {
                pts[0].y
            }
        } else if x >= pts[n - 1].x {
            // Linear ramp from the last point to (1,1)
            if pts[n - 1].x < 1.0 {
                let t = (x - pts[n - 1].x) / (1.0 - pts[n - 1].x);
                pts[n - 1].y + t * (1.0 - pts[n - 1].y)
            } else {
                pts[n - 1].y
            }
        } else {
            // Find segment
            let mut i = 0;
            while i < n - 1 && x > pts[i + 1].x {
                i += 1;
            }

            let h = pts[i + 1].x - pts[i].x;
            let t = (x - pts[i].x) / h;
            
            // Hermite basis functions
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;

            h00 * pts[i].y + h10 * h * m[i] + h01 * pts[i + 1].y + h11 * h * m[i + 1]
        };

        lut[x_idx] = (y.clamp(0.0, 1.0) * 255.0) as u8;
    }

    lut
}