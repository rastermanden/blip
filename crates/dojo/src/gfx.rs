use blip::image::Image;
use blip::Asset;

fn dojo() -> Vec<u8> {
    let mut img = Image::new(480, 540);
    for y in 0..540 {
        for x in 0..480 {
            if y >= 430 {
                let plank = ((y - 430) / 16) % 2 == 0;
                let seam = x % 80 < 2;
                let (mut r, mut g, mut b) = if plank { (132, 82, 46) } else { (108, 66, 36) };
                if seam {
                    r -= 20;
                    g -= 12;
                    b -= 8;
                }
                img.set(x, y, r, g, b);
            } else if y >= 300 {
                // A dark dado, so a white gi reads against the wall.
                let post = x % 160 < 8;
                let (r, g, b) = if post { (58, 32, 26) } else { (92, 40, 34) };
                img.set(x, y, r, g, b);
            } else {
                let band = (y / 48) % 2 == 0;
                let v = if band { 214 } else { 206 };
                img.set(x, y, v, v - 16, v - 38);
            }
        }
    }
    // Three paper screens, dark wood lattice.
    for (px, py) in [(28, 64), (176, 64), (324, 64)] {
        for y in py..py + 230 {
            for x in px..px + 128 {
                let edge = x == px || x == px + 127 || y == py || y == py + 229;
                let lattice = (x - px) % 32 == 0 || (y - py) % 46 == 0;
                if edge || lattice {
                    img.set(x, y, 86, 58, 36);
                } else if (x + y) % 17 == 0 {
                    img.set(x, y, 228, 216, 190);
                } else {
                    img.set(x, y, 240, 230, 208);
                }
            }
        }
    }
    for y in 412..432 {
        for x in 0..480 {
            img.set(x, y, 74, 46, 28);
        }
    }
    img.encode_png()
}

fn yard() -> Vec<u8> {
    let mut img = Image::new(480, 540);
    for y in 0..540 {
        for x in 0..480 {
            if y < 392 {
                let t = y as f32 / 392.0;
                let r = (32.0 + t * 188.0) as u8;
                let g = (40.0 + t * 70.0) as u8;
                let b = (92.0 - t * 28.0) as u8;
                img.set(x, y, r, g, b);
            } else {
                let speck = (x / 5 + y * 3) % 11 == 0;
                img.set(x, y, if speck { 128 } else { 112 }, if speck { 92 } else { 78 }, 52);
            }
        }
    }
    let (cx, cy, rad) = (348, 286, 34);
    for y in (cy - rad)..(cy + rad) {
        for x in (cx - rad)..(cx + rad) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= rad * rad {
                img.set(x, y, 255, 206, 120);
            }
        }
    }
    for post in (24..470).step_by(32) {
        for y in 292..404 {
            img.set(post, y, 72, 52, 36);
            img.set(post + 1, y, 88, 64, 42);
        }
    }
    for rail in [328, 364] {
        for y in rail..rail + 5 {
            for x in 16..468 {
                img.set(x, y, 96, 70, 44);
            }
        }
    }
    img.encode_png()
}

pub fn generate() -> Vec<Asset> {
    vec![("images/dojo.png", dojo()), ("images/yard.png", yard())]
}
