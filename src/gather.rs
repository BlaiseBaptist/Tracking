pub mod feed {
    use rscam::{Camera, Config, Frame};
    use std::collections::HashMap;
    use std::sync::mpsc::{channel, Sender};
    use std::thread;
    use std::time::{Duration, Instant};
    pub type ImageDiff = Vec<u8>;
    type CFrame = (Frame, usize);
    pub const FRAME_RATE: u32 = 30;
    pub const RESOLUTION: (u32, u32) = (1920, 1080);
    pub const FORMAT: [u8; 4] = *b"RGB3";
    const CONFIG: Config = Config {
        interval: (1, FRAME_RATE),
        resolution: RESOLUTION,
        format: &FORMAT,
        field: rscam::FIELD_NONE,
        nbuffers: 32,
    };
    pub fn start_cameras(cameras: Vec<&str>, sender: Sender<Vec<ImageDiff>>) {
        let (c_frame_send, c_frame_receive) = channel::<CFrame>();
        let num_cameras = cameras.len();
        cameras
            .into_iter()
            .enumerate()
            .for_each(|v| start_camera(v.0, v.1.to_string(), c_frame_send.clone()));
        let mut old_frames: HashMap<usize, Frame> = HashMap::new();
        loop {
            let now = Instant::now();
            let status = sender.send(
                (0..num_cameras)
                    .filter_map(|_| match c_frame_receive.try_recv() {
                        Ok(new_c_frame) => get_diff(&mut old_frames, new_c_frame).ok(),

                        Err(_) => None,
                    })
                    .collect(),
            );
            match status {
                Ok(_) => {}
                Err(_) => return,
            }
            let processing_time = now.elapsed();
            thread::sleep((Duration::from_secs(1) / FRAME_RATE).saturating_sub(processing_time))
        }
    }
    fn get_diff(
        old_frames: &mut HashMap<usize, Frame>,
        new_c_frame: CFrame,
    ) -> Result<ImageDiff, String> {
        let key = new_c_frame.1.clone();
        return match old_frames.insert(new_c_frame.1, blur(new_c_frame.0)) {
            Some(old_frame) => Ok(compare_frames(
                old_frames.get(&key).ok_or("No frame")?,
                &old_frame,
            )),

            None => Err("New Camera, No first frame".to_string()),
        };
    }
    fn blur(to_blur: Frame) -> Frame {
        //     te6b39 mut image use=
        //         image::ImageBuffer::from_vec(RESOLUTION.0, RESOLUTION.1, to_blur.iter().collect());
        to_blur
    }
    fn start_camera(id: usize, path: String, sender: Sender<CFrame>) {
        thread::spawn(move || {
            let mut camera = Camera::new(&path).unwrap();

            //Backlight Compensation
            camera.set_control(9963804, &3).unwrap();

            //White Balance, Automatic
            camera.set_control(9963788, &true).unwrap();

            //Auto Exposure
            camera.set_control(10094849, &3).unwrap();

            //Exposure Time, Absolute
            // camera.set_control(10094850, &380).unwrap();

            //Focus, Automatic Continuous
            camera.set_control(10094860, &true).unwrap();

            //Focus, Absolute
            // camera.set_control(10094858, &1).unwrap();

            print_cam_options(&camera);
            camera.start(&CONFIG).unwrap();

            loop {
                match sender.send((camera.capture().unwrap(), id)) {
                    Ok(_) => {}
                    Err(v) => {
                        println!("{:?}", v);
                        break;
                    }
                }
            }
        });
    }
    fn compare(v1: [u8; 3], v2: [u8; 3]) -> u8 {
        value_scale(
            v1.iter()
                .zip(v2.iter())
                .fold(0.0, |acc: f32, (&v1, &v2)| acc + (v1 as f32 - v2 as f32)),
        )
    }
    fn value_scale(v: f32) -> u8 {
        // let new = if v.abs() > 100.0 { 255 } else { 0 };
        // return new;
        return (v / 3.0) as u8;
    }
    fn compare_frames(frame1: &Frame, frame2: &Frame) -> ImageDiff {
        let return_val = frame1
            .chunks(3)
            .zip(frame2.chunks(3))
            .map(|v| compare((*v.0).try_into().unwrap(), (*v.1).try_into().unwrap()))
            .collect::<ImageDiff>();
        let num_rays = return_val.iter().fold(0.0, |acc, &e| acc + e as f32) / 255.0;
        println!("num rays: {}", num_rays);
        return_val
    }
    fn print_cam_options(camera: &rscam::Camera) {
        let formats: Vec<_> = camera.formats().collect();

        let reses = match camera.resolutions(FORMAT).unwrap() {
            rscam::ResolutionInfo::Discretes(v) => v,
            _ => todo!(),
        };
        let _: Vec<_> = camera
            .controls()
            .inspect(|val| {
                println!(
                    "{} {}:{:?}",
                    val.as_ref().unwrap().id,
                    val.as_ref().unwrap().name,
                    val.as_ref().unwrap().data,
                )
            })
            .collect();
        let intervals: Vec<_> = reses
            .into_iter()
            .map(|res| match camera.intervals(FORMAT, res).unwrap() {
                rscam::IntervalInfo::Discretes(v) => (res, v[0].1),
                _ => todo!(),
            })
            .collect();
        println!("{:?}", formats);
        for interval in intervals {
            println!("res: {:?}, fps: {}", interval.0, interval.1);
        }
    }
}
