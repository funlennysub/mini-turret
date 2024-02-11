use opencv::core::{bitwise_and, flip, in_range, Point, Scalar, Size, BORDER_CONSTANT};
use opencv::imgproc::{
    bounding_rect, circle, contour_area, cvt_color, find_contours, get_structuring_element,
    moments, morphology_default_border_value, morphology_ex, put_text, rectangle, resize,
    CHAIN_APPROX_SIMPLE, COLOR_BGR2HSV, FONT_HERSHEY_PLAIN, INTER_AREA, LINE_8, MORPH_CLOSE,
    MORPH_ELLIPSE, RETR_EXTERNAL,
};

use opencv::types::VectorOfVectorOfPoint;
use opencv::{
    prelude::{Mat, MatTraitConst, MatTraitConstManual, VideoCaptureTrait},
    videoio::{self, VideoCapture},
};

#[derive(Default)]
pub struct Vision {
    pub source: Option<VideoCapture>,
    contours: VectorOfVectorOfPoint,
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

    pub fn disconnect(&mut self) -> crate::Result<()> {
        if let Some(src) = &mut self.source {
            src.release()?;
        }

        Ok(())
    }

    pub fn get_frame(&mut self, flip_frame: bool) -> crate::Result<Mat> {
        let mut frame = Mat::default();
        if let Some(src) = &mut self.source {
            src.read(&mut frame)?;
        }

        let new = if flip_frame {
            let mut flipped = Mat::default();
            flip(&frame, &mut flipped, 1)?;
            flipped
        } else {
            frame
        };

        let mut resized = Mat::default();
        resize(&new, &mut resized, Size::new(640, 480), 0., 0., INTER_AREA)?;

        Ok(resized)
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

        let kernel = get_structuring_element(MORPH_ELLIPSE, Size::new(10, 10), Point::new(-1, -1))?;
        let mut morph = Mat::default();

        morphology_ex(
            &mask,
            &mut morph,
            MORPH_CLOSE,
            &kernel,
            Point::new(-1, -1),
            2,
            BORDER_CONSTANT,
            morphology_default_border_value()?,
        )?;

        Ok(morph)
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

        self.contours = contours;

        Ok(())
    }

    pub fn draw_bb(&self, img: &Mat, min_size: f64) -> crate::Result<Mat> {
        let mut out = Mat::clone(&img);

        for contour in &self.contours {
            let area = contour_area(&contour, false)?;
            if area > min_size {
                let a = bounding_rect(&contour)?;
                rectangle(&mut out, a, Scalar::new(0., 0., 255., 0.), 3, LINE_8, 0)?;
                let moments = moments(&contour, false)?;

                let color = Scalar::new(230., 255., 255., 0.);
                let center = (
                    (moments.m10 / moments.m00) as i32,
                    (moments.m01 / moments.m00) as i32,
                );

                circle(
                    &mut out,
                    Point::new(center.0, center.1),
                    5,
                    color,
                    -1,
                    LINE_8,
                    0,
                )?;
                put_text(
                    &mut out,
                    &format!("{}, {} - {area}", center.0, center.1),
                    Point::new(a.x, a.y - 10),
                    FONT_HERSHEY_PLAIN,
                    1.,
                    color,
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
