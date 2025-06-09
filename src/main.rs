use rscam::{Camera, Config};
use std::fs::File;
use std::io::Write;
fn main() {
    let mut camera = Camera::new("/dev/video0").unwrap();

    camera
        .start(&Config {
            interval: (1, 30), // 30 fps.
            resolution: (2560, 1440),
            format: b"YUYV",
            ..Default::default()
        })
        .unwrap();
    let format = b"YUYV";
    let reses = match camera.resolutions(format).unwrap() {
        rscam::ResolutionInfo::Discretes(v) => v,
        _ => todo!(),
    };
    let intervals: Vec<_> = reses
        .into_iter()
        .map(|res| match camera.intervals(format, res).unwrap() {
            rscam::IntervalInfo::Discretes(v) => (v[0], res),
            _ => todo!(),
        })
        .collect();
    println!("{}", String::from_utf8(format.clone().to_vec()).unwrap());
    for interval in intervals {
        println!("{:?}", interval);
    }
    let frame = camera.capture().unwrap();
    let mut file = File::create("frame").unwrap();
    file.write(&frame).unwrap();
}
