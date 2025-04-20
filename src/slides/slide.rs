use crate::{
    err_new_image,
    error::Result,
    imageproc::{
        drawing::{DrawMut, DrawText},
        rect::Rect,
    },
};
use ab_glyph::FontArc;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use std::path::PathBuf;

#[derive(Clone)]
pub enum ContentType {
    Image(PathBuf),
    Text {
        content: String,
        max_scale: f32,
        color: Rgba<u8>,
        background_color: Option<Rgba<u8>>,
        font: FontArc,
    },
}

#[derive(Clone)]
pub struct SlideElement {
    content: ContentType,
    position: Rect,
}

#[derive(Clone)]
pub struct Slide {
    elements: Vec<SlideElement>,
    background: Option<PathBuf>,
}

impl Default for Slide {
    fn default() -> Self {
        Slide {
            elements: vec![SlideElement {
                content: ContentType::Text {
                    content: "default".to_string(),
                    max_scale: 100.0,
                    color: [255, 0, 0, 255].into(),
                    background_color: None,
                    font: FontArc::try_from_slice(include_bytes!("../MiSans-Demibold.ttf"))
                        .unwrap(),
                },
                position: Rect::at(0, 0).of_size(480, 500),
            }],
            background: None,
        }
    }
}

pub fn render_frame(slide: &Slide) -> Result<DynamicImage> {
    let mut img = match slide.background {
        Some(ref path) => image::open(path).unwrap_or_else(|_| DynamicImage::new_rgba8(1920, 1080)),
        _ => DynamicImage::new_rgba8(480, 1080),
    };
    for element in &slide.elements {
        let rect = element.position;
        match element.content {
            ContentType::Image(ref path) => {
                let img_element = image::open(path)
                    .map_err(|e| err_new_image!(e))?
                    .thumbnail(rect.width(), rect.height());
                let (img_w, img_h) = img_element.dimensions();
                img.copy_from(
                    &img_element,
                    rect.left() as u32 + (rect.width() - img_w) / 2,
                    rect.top() as u32 + (rect.height() - img_h) / 2,
                )
                .map_err(|e| err_new_image!(e))?;
            }
            ContentType::Text {
                ref content,
                max_scale,
                color,
                background_color,
                ref font,
            } => {
                if let Some(background_color) = background_color {
                    img.draw_filled_rounded_rect_mut(rect, 10, background_color);
                }
                img.draw_text_center_mut(color, rect, max_scale, &font, &content);
            }
        }
    }
    // 绘制分割线
    img.draw_line_segment_mut((0.0, 10.0), (0.0, 1080.0), Rgba([255, 0, 0, 255]));

    img.save("./src/output.png")
        .map_err(|e| err_new_image!(e))?;
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_name() {
        let font = FontArc::try_from_slice(include_bytes!("../MiSans-Demibold.ttf")).unwrap();
        let slides = Slide {
            elements: vec![
                SlideElement {
                    content: ContentType::Image(PathBuf::from("D:/pictures/arknights\\奥达_2.png")),
                    position: Rect::at(0, 0).of_size(480, 500),
                },
                SlideElement {
                    content: ContentType::Text {
                        content: "this is test 1".to_string(),
                        max_scale: 100.0,
                        color: [255, 0, 0, 255].into(),
                        background_color: Some([23, 150, 235, 255].into()),
                        font: font.clone(),
                    },
                    position: Rect::at(10, 600).of_size(460, 150),
                },
                SlideElement {
                    content: ContentType::Text {
                        content: "oh!\nthis is test 2".to_string(),
                        max_scale: 100.0,
                        color: [0, 255, 0, 255].into(),
                        background_color: Some([44, 85, 153, 255].into()),
                        font: font.clone(),
                    },
                    position: Rect::at(10, 800).of_size(460, 150),
                },
            ],
            background: None,
        };
        let _ = render_frame(&slides).unwrap();
    }
}
