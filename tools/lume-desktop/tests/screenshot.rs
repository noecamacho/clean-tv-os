//! The harness renders the empty screen to PNG files at both reference resolutions.

use lume_desktop::screenshot;
use std::path::PathBuf;

fn temp_png(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("lume-test-{}-{name}.png", std::process::id()))
}

fn decode(path: &PathBuf) -> (png::OutputInfo, Vec<u8>) {
    let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buffer).unwrap();
    (info, buffer)
}

#[test]
fn empty_screen_at_720p_and_1080p() {
    for (w, h) in [(1280u32, 720u32), (1920, 1080)] {
        let path = temp_png(&format!("{w}x{h}"));
        match screenshot::save(&path, w, h, screenshot::draw_empty) {
            Ok(()) => {}
            Err(e) if e.contains("EGL") || e.contains("GL context") => {
                eprintln!("skipped: {e}");
                return;
            }
            Err(e) => panic!("{e}"),
        }
        let (info, pixels) = decode(&path);
        assert_eq!((info.width, info.height), (w, h));
        // Every pixel is the theme background (#0F171E).
        assert!(
            pixels.chunks(4).all(|p| p == [0x0F, 0x17, 0x1E, 255]),
            "{w}x{h} is not a flat background"
        );
        std::fs::remove_file(path).unwrap();
    }
}
