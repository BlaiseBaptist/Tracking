pub mod feed {
    use rscam::{Camera, Config, CtrlData, Frame};
    use std::sync::mpsc::{channel, Receiver, Sender};
    pub struct Cameras {
        sender: Sender<Vec<Vec<u8>>>,
        pub receiver: Option<Receiver<Vec<Vec<u8>>>>, //vec differences
        cameras: Vec<Receiver<Frame>>,
    }
    impl Cameras {
        fn new(cameras: Vec<Camera>) -> Self {
            let (sender, receiver) = channel::<Vec<Vec<u8>>>();
            let async_cameras = cameras.into_iter().map(|v| into_async(v)).collect();
            Cameras {
                sender,
                receiver: Some(receiver),
                cameras: async_cameras,
            }
        }
    }
    fn into_async(camera: Camera) -> Receiver<Frame> {
        let (sender, receiver) = channel::<Frame>();
        std::thread::spawn(move || loop {
            match sender.send(camera.capture().unwrap()) {
                Err(_) => break,
                Ok(_) => {}
            }
        });
        receiver
    }
}
