use opencv::imgproc::{cvt_color, COLOR_BGR2RGBA};
use opencv::prelude::{Mat, MatTraitConst, MatTraitConstManual, VideoCaptureTrait};
use opencv::videoio;
use opencv::videoio::VideoCapture;

#[derive(Default)]
pub struct Camera {
    pub source: Option<VideoCapture>,
}

impl Camera {
    // TODO: list cameras and connect by id
    pub fn connect(&mut self, camera_id: i32) -> crate::Result<()> {
        self.source = Some(VideoCapture::new(camera_id, videoio::CAP_ANY)?);

        Ok(())
    }

    pub fn get_frame(&mut self) -> crate::Result<([usize; 2], Vec<u8>)> {
        let mut frame = Mat::default();
        if let Some(src) = &mut self.source {
            src.read(&mut frame)?;
        }

        let mut rgba_frame = Mat::default();
        cvt_color(&frame, &mut rgba_frame, COLOR_BGR2RGBA, 0)?;
        assert!(rgba_frame.is_continuous());

        Ok((
            [rgba_frame.cols() as usize, rgba_frame.rows() as usize],
            rgba_frame.data_bytes()?.into(),
        ))
    }
}
