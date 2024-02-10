use opencv::core::{bitwise_and, in_range, Moments, Point, Scalar, Size, BORDER_DEFAULT};
use opencv::imgproc::{
    blur, bounding_rect, contour_area, find_contours, moments, put_text, rectangle,
    CHAIN_APPROX_SIMPLE, COLOR_BGR2HSV, FONT_HERSHEY_PLAIN, LINE_8, RETR_EXTERNAL,
};

use opencv::types::VectorOfVectorOfPoint;
use opencv::{
    imgproc::cvt_color,
    prelude::{Mat, MatTraitConst, MatTraitConstManual, VideoCaptureTrait},
    videoio::{self, VideoCapture},
};

#[derive(Default)]
pub struct Vision {
    pub source: Option<VideoCapture>,
    contours: VectorOfVectorOfPoint,
    moments: Option<Moments>,
}

pub fn to_rgba(frame: &Mat, code: i32) -> crate::Result<Mat> {
    let mut rgba_frame = Mat::default();
    cvt_color(&frame, &mut rgba_frame, code, 0)?;

    Ok(rgba_frame)
}

pub fn mat_size_and_vec(mat: &Mat) -> crate::Result<([usize; 2], Vec<u8>)> {
    assert!(mat.is_continuous());

    Ok((
        [mat.cols() as usize, mat.rows() as usize],
        mat.data_bytes()?.into(),
    ))
}

impl Vision {
    // TODO: list cameras and connect by id
    pub fn connect(&mut self, camera_id: i32) -> crate::Result<()> {
        self.source = Some(VideoCapture::new(camera_id, videoio::CAP_ANY)?);

        Ok(())
    }

    pub fn get_frame(&mut self) -> crate::Result<Mat> {
        let mut frame = Mat::default();
        if let Some(src) = &mut self.source {
            src.read(&mut frame)?;
        }

        Ok(frame)
    }

    /// Returns GRAY blurred Mat
    pub fn filter_color(
        &self,
        src: &Mat,
        lower_bound: (u8, u8, u8),
        upper_bound: (u8, u8, u8),
    ) -> crate::Result<Mat> {
        let mut hsv_frame = Mat::default();
        cvt_color(&src, &mut hsv_frame, COLOR_BGR2HSV, 0)?;

        let lower = Mat::from_slice(&[lower_bound.0, lower_bound.1, lower_bound.2])?;
        let upper = Mat::from_slice(&[upper_bound.0, upper_bound.1, upper_bound.2])?;

        let mut mask = Mat::default();
        in_range(&hsv_frame, &lower, &upper, &mut mask)?;

        let mut blurred = Mat::default();
        blur(
            &mask,
            &mut blurred,
            Size::new(3, 3),
            Point::new(-1, -1),
            BORDER_DEFAULT,
        )?;

        Ok(blurred)
    }

    pub fn get_contours(&mut self, gray_mat: &Mat) -> crate::Result<()> {
        let mut contours = VectorOfVectorOfPoint::new();
        find_contours(
            &gray_mat,
            &mut contours,
            RETR_EXTERNAL,
            CHAIN_APPROX_SIMPLE,
            Point::default(),
        )?;

        self.moments = Some(moments(&gray_mat, true)?);

        self.contours = contours;

        Ok(())
    }

    pub fn draw_bb(&self, img: &Mat, min_size: f64) -> crate::Result<Mat> {
        let mut out = Mat::copy(img)?;

        for contour in &self.contours {
            if contour_area(&contour, false)? > min_size {
                let a = bounding_rect(&contour)?;
                rectangle(&mut out, a, Scalar::new(0., 0., 255., 0.), 3, LINE_8, 0)?;
                let m = self.moments;

                put_text(
                    &mut out,
                    &m.map(|m| format!("{:.0}, {:.0}", m.m10 / m.m00, m.m01 / m.m00))
                        .unwrap_or_default(),
                    Point::new(a.x, a.y - 10),
                    FONT_HERSHEY_PLAIN,
                    1.,
                    Scalar::new(230., 255., 255., 0.),
                    2,
                    LINE_8,
                    false,
                )?;
            }
        }

        Ok(out)
    }

    pub fn combine(&self, src: &Mat, mask: &Mat) -> crate::Result<Mat> {
        let mut and = Mat::default();
        bitwise_and(&src, &src, &mut and, &mask)?;

        Ok(and)
    }
}
