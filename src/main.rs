mod gather;
use gather::feed;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
// mod imageprocessing;
fn main() {
    let path1 = "/dev/video0";
    let (sender, cameras) = channel::<Vec<feed::ImageDiff>>();
    thread::spawn(move || feed::start_cameras(vec![path1], sender));
    let start = Instant::now();
    let mut frame_count = 0;
    // let max_frames = feed::FRAME_RATE * 5;
    let max_frames = 300;
    while frame_count < max_frames {
        let diffs = cameras.recv().unwrap();
        frame_count += diffs.len() as u32;
        diffs.iter().for_each(|diff| {
            image::save_buffer_with_format(
                &format!("../images/diff_{:05}.png", frame_count),
                &diff,
                gather::feed::RESOLUTION.0,
                gather::feed::RESOLUTION.1,
                image::ColorType::L8,
                image::ImageFormat::Png,
            )
            .unwrap();
        });
        // println!("{:05}", i);
    }
    let framerate = frame_count as f64 / start.elapsed().as_secs_f64();
    println!(
        "took {:?} to capture {} frames at {} fps",
        start.elapsed(),
        frame_count,
        framerate
    );
    Command::new("ffmpeg")
        .args([
            "-framerate",
            &framerate.to_string(),
            "-i",
            "images/diff_%5d.png",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            &format!(
                "outputs/output{}.mp4",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        ])
        .output()
        .unwrap();
}
