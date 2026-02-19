use image::GenericImageView;

fn main() {
    let img_new = image::open("vibe_dither_exportNew.png").expect("Failed to open New");
    let img_old = image::open("vibe_dither_exportOld.png").expect("Failed to open Old");
    let img_truth = image::open("vibe_dither_InAppScreenshot.png").expect("Failed to open Truth");

    // Find first non-black pixel in Truth to compare
    let (tw, th) = img_truth.dimensions();
    let (nw, nh) = img_new.dimensions();
    
    // Scale coordinates
    let scale_x = nw as f32 / tw as f32;
    let scale_y = nh as f32 / th as f32;

    println!("Scaling: {}x, {}y", scale_x, scale_y);

    let mut found = false;
    for y in (0..th).step_by(5) {
        for x in (0..tw).step_by(5) {
            let p_t = img_truth.get_pixel(x, y);
            if p_t[0] > 10 || p_t[1] > 10 || p_t[2] > 10 {
                let nx = (x as f32 * scale_x) as u32;
                let ny = (y as f32 * scale_y) as u32;
                
                if nx < nw && ny < nh {
                    let p_n = img_new.get_pixel(nx, ny);
                    let p_o = img_old.get_pixel(nx, ny);
                    
                    println!("Pixel at Truth({},{}), New({},{}):", x, y, nx, ny);
                    println!("  Truth: {:?}", p_t);
                    println!("  New:   {:?}", p_n);
                    println!("  Old:   {:?}", p_o);
                    found = true;
                    break;
                }
            }
        }
        if found { break; }
    }
}
