use rscam::{Camera, Config, CtrlData, Frame};
#[allow(dead_code)]
fn print_cam_options(camera: &rscam::Camera) {
    let format = b"RGB3";
    // let formats: Vec<_> = camera.formats().collect();
    let _: Vec<_> = camera
        .controls()
        .inspect(|val| {
            println!(
                "id: {}, name: {}, flags: {}",
                val.as_ref().unwrap().id,
                val.as_ref().unwrap().name,
                val.as_ref().unwrap().flags
            )
        })
        .collect();

    match camera.get_control(9963788).unwrap().data {
        CtrlData::Boolean {
            value: v,
            default: d,
        } => println!("{},{}", v, d),

        _ => panic!(),
    }
    let reses = match camera.resolutions(format).unwrap() {
        rscam::ResolutionInfo::Discretes(v) => v,
        _ => todo!(),
    };
    let intervals: Vec<_> = reses
        .into_iter()
        .map(|res| match camera.intervals(format, res).unwrap() {
            rscam::IntervalInfo::Discretes(v) => (res, v[0].1),
            _ => todo!(),
        })
        .collect();
    println!("{}", String::from_utf8(format.clone().to_vec()).unwrap());
    for interval in intervals {
        println!("res: {:?}, fps: {}", interval.0, interval.1);
    }
}
fn compare(v1: [u8; 3], v2: [u8; 3]) -> [u8; 3] {
    [(v1.iter().fold(0, |acc: usize, &e| acc + e as usize) as f32 / 3.0
        - v2.iter().fold(0, |acc: usize, &e| acc + e as usize) as f32 / 3.0)
        .round() as u8; 3]
}
fn compare_frames(frame1: &Frame, frame2: &Frame) -> Vec<u8> {
    frame1
        .chunks(3)
        .zip(frame2.chunks(3))
        .flat_map(|v| compare((*v.0).try_into().unwrap(), (*v.1).try_into().unwrap()))
        .collect()
}
fn main() {
    let mut camera = Camera::new("/dev/video0").unwrap();
    #[cfg(debug_assertions)]
    print_cam_options(&camera);
    let res = (1920, 1080);
    camera
        .start(&Config {
            interval: (1, 60),
            resolution: res,
            format: b"RGB3",
            ..Default::default()
        })
        .unwrap();

    let mut oldframe = camera.capture().unwrap();

    for i in 0..1800 {
        let newframe = camera.capture().unwrap();

        let diff = compare_frames(&oldframe, &newframe); // Vec<u8>

        image::save_buffer_with_format(
            &format!("images/diff_{:05}.png", i),
            &diff,
            res.0,
            res.1,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .unwrap();
        println!("{:05}", i);
        oldframe = newframe;
    }
}
