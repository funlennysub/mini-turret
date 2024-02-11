use opencv::core::{flip, in_range, Point, Scalar, Size, BORDER_CONSTANT, BORDER_DEFAULT};
use opencv::imgproc::{
    circle, contour_area, cvt_color, find_contours, gaussian_blur, get_structuring_element,
    moments, morphology_default_border_value, morphology_ex, put_text, resize, CHAIN_APPROX_SIMPLE,
    COLOR_BGR2HSV, FONT_HERSHEY_PLAIN, INTER_AREA, LINE_8, MORPH_CLOSE, MORPH_ELLIPSE, MORPH_OPEN,
    RETR_EXTERNAL,
};
use std::fmt::{Display, Formatter};

use opencv::types::VectorOfVectorOfPoint;
use opencv::{
    prelude::{Mat, MatTraitConst, MatTraitConstManual, VideoCaptureTrait},
    videoio::{self, VideoCapture},
};

// TODO: maybe run vision on separate thread so app is usable in the meantime

const NEG_POINT: Point = Point::new(-1, -1);

struct Target {
    x: i32,
    y: i32,
    area: f64,
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {} :: {}", self.x, self.y, self.area)
    }
}

#[derive(Default)]
pub struct Vision {
    pub source: Option<VideoCapture>,
    contours: VectorOfVectorOfPoint,

    targets: Vec<Target>,
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

    // TODO: maybe find a better way to denoise
    /// Returns GRAY blurred Mat
    pub fn filter_color(
        &self,
        src: &Mat,
        lower_bound: (u8, u8, u8),
        upper_bound: (u8, u8, u8),
    ) -> crate::Result<Mat> {
        let mut gb = Mat::default();
        gaussian_blur(&src, &mut gb, Size::new(15, 15), 0., 0., BORDER_DEFAULT)?;

        let mut hsv_frame = Mat::default();
        cvt_color(&gb, &mut hsv_frame, COLOR_BGR2HSV, 0)?;

        let lower = Mat::from_slice(&[lower_bound.0, lower_bound.1, lower_bound.2])?;
        let upper = Mat::from_slice(&[upper_bound.0, upper_bound.1, upper_bound.2])?;

        let mut mask = Mat::default();
        in_range(&hsv_frame, &lower, &upper, &mut mask)?;

        let kernel_close = get_structuring_element(MORPH_ELLIPSE, Size::new(3, 3), NEG_POINT)?;
        let kernel_open = get_structuring_element(MORPH_ELLIPSE, Size::new(7, 7), NEG_POINT)?;

        let mut morph_open = Mat::default();
        morphology_ex(
            &mask,
            &mut morph_open,
            MORPH_OPEN,
            &kernel_open,
            NEG_POINT,
            2,
            BORDER_CONSTANT,
            morphology_default_border_value()?,
        )?;

        let mut morph_close = Mat::default();
        morphology_ex(
            &morph_open,
            &mut morph_close,
            MORPH_CLOSE,
            &kernel_close,
            NEG_POINT,
            4,
            BORDER_CONSTANT,
            morphology_default_border_value()?,
        )?;

        Ok(morph_close)
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

    // TODO: ignore points inside same contour
    pub fn find_targets(&mut self, min_size: f64) -> crate::Result<()> {
        let mut targets = Vec::new();
        for c in &self.contours {
            let area = contour_area(&c, false)?;
            if area < min_size {
                continue;
            }

            let moments = moments(&c, false)?;

            targets.push(Target {
                x: (moments.m10 / moments.m00) as i32,
                y: (moments.m01 / moments.m00) as i32,
                area,
            });
        }

        self.targets = targets;

        Ok(())
    }

    pub fn display_info(&self, img: &Mat) -> crate::Result<Mat> {
        let mut out = Mat::clone(img);
        let color = Scalar::new(230., 255., 255., 0.);

        for target in &self.targets {
            circle(
                &mut out,
                Point::new(target.x, target.y),
                5,
                color,
                -1,
                LINE_8,
                0,
            )?;
            put_text(
                &mut out,
                &target.to_string(),
                Point::new(target.x, target.y - 10),
                FONT_HERSHEY_PLAIN,
                1.,
                color,
                2,
                LINE_8,
                false,
            )?;
        }

        Ok(out)
    }
}
