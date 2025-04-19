#![allow(unused)]
use super::imageproc::{
    drawing::{DrawMut, DrawText},
    rect::Rect,
};
use super::BigImg;
use crate::{
    err_new, err_new_image,
    error::{Kind, Result},
};
use ab_glyph::FontArc;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

struct VideoConfig {
    screen: (u32, u32),
    fps: u32,
    transition_sec: u32,
    output_path: PathBuf,
}

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

struct SlideElement {
    content: ContentType,
    position: Rect,
}

struct Slide {
    elements: Vec<SlideElement>,
    duration_sec: u32,
    background: Option<PathBuf>,
}

fn reader_frame(slide: Slide) -> Result<DynamicImage> {
    let mut img = match slide.background {
        Some(ref path) => image::open(path).unwrap_or_else(|_| DynamicImage::new_rgba8(1920, 1080)),
        _ => DynamicImage::new_rgba8(480, 1080),
    };
    for element in slide.elements {
        let rect = element.position;
        match element.content {
            ContentType::Image(path) => {
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
                content,
                max_scale,
                color,
                background_color,
                font,
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

// 定义 Chunk 结构体
// pub struct Chunk(Vec<Area>);

// 定义 Chunk 结构体
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pic_path: PathBuf,
    text_up: Vec<String>,
    text_down: Vec<String>,
}

// 实现 Chunk 结构体的 Debug trait
impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pic_path", &self.pic_path.to_str())
            .field("text_up", &self.text_up)
            .field("text_down", &self.text_down)
            .finish()
    }
}

impl Chunk {
    /// 创建一个新的 Chunk 实例
    ///
    /// # Parameters
    ///
    /// * `pic_path` - 图片文件的路径
    /// * `text_up` - 上方文本的向量
    /// * `text_down` - 下方文本的向量
    ///
    /// # Returns
    ///
    /// 如果图片路径有效，则返回一个包含给定数据的 Chunk 实例；否则返回错误。
    ///
    /// # Errors
    ///
    /// * `IoError(NotFound)` - 如果提供的图片路径无效或不存在
    pub fn new(pic_path: PathBuf, text_up: Vec<String>, text_down: Vec<String>) -> Result<Self> {
        if !pic_path.exists() {
            return Err(err_new!(
                Kind::IoError(std::io::ErrorKind::NotFound),
                "Invalid path"
            ));
        }
        Ok(Chunk {
            pic_path,
            text_up,
            text_down,
        })
    }

    /// 绘制 Chunk 数据到一个图像上
    ///
    /// # Parameters
    ///
    /// * `si` - 包含屏幕信息和样式的大图像实例
    ///
    /// # Returns
    ///
    /// 返回一个包含绘制数据的 `DynamicImage` 实例。
    ///
    /// # Errors
    ///
    /// * `ImageError` - 如果打开或处理图片时发生错误
    /// * `TryFromIntError` - 如果在类型转换过程中发生溢出或其他错误
    pub fn draw_data(&self, si: &BigImg) -> Result<DynamicImage> {
        // 解构 BigImg 实例，获取所需的字段
        let BigImg {
            screen,
            width_chunk,
            text_background_color,
            text_color,
            max_scale,
            pic_h,
            text_up_h,
            text_down_h,
            font,
            ..
        } = si;

        // 创建一个新的 `DynamicImage` 实例作为绘制目标
        let mut target = DynamicImage::new_rgba8(*width_chunk, screen.1);

        // 打开并调整图片大小
        let img = image::open(&self.pic_path)
            .map_err(|e| err_new_image!(e))?
            .thumbnail(*width_chunk, *pic_h);
        let (img_w, img_h) = img.dimensions();
        // 将调整好大小的图片复制到目标图像的中心位置
        target
            .copy_from(&img, (width_chunk - img_w) / 2, (pic_h - img_h) / 2)
            .map_err(|e| err_new_image!(e))?;

        // 绘制上下文本的背景框
        let text_up_rect = Rect::at(1, i32::try_from(*pic_h)?).of_size(width_chunk - 1, *text_up_h);
        let text_down_rect =
            Rect::at(1, i32::try_from(pic_h + text_up_h)?).of_size(width_chunk - 1, *text_down_h);
        target.draw_filled_rounded_rect_mut(text_up_rect, 10, text_background_color.0);
        target.draw_filled_rounded_rect_mut(text_down_rect, 10, text_background_color.1);

        // 获取上下文本的内容和长度
        let (text_up, text_down) = (&self.text_up, &self.text_down);
        let (len_up, len_down) = (text_up.len(), text_down.len());
        let (h_up, h_down) = (
            text_up_h / u32::try_from(len_up)?,
            (text_down_h - 30) / u32::try_from(len_down)?,
        );

        // 绘制上文本
        for (i, str) in text_up.iter().enumerate() {
            let high = pic_h + u32::try_from(i)? * h_up;
            target.draw_text_center_mut(
                *text_color,
                Rect::at(10, i32::try_from(high)?).of_size(width_chunk - 20, h_up),
                *max_scale,
                &font,
                str,
            );
        }

        // 绘制下文本
        for (i, str) in text_down.iter().enumerate() {
            let high = pic_h + text_up_h + u32::try_from(i)? * h_down;
            target.draw_text_center_mut(
                *text_color,
                Rect::at(10, i32::try_from(high)?).of_size(width_chunk - 20, h_down),
                *max_scale,
                &font,
                str,
            );
            // target.text_center(
            //     *text_color,
            //     Rect::at(10, i32::try_from(high)?).of_size(width_chunk - 20, h_down),
            //     *max_scale,
            //     &font,
            //     str,
            // );
        }

        // 绘制分割线
        target.draw_line_segment_mut((0.0, 10.0), (0.0, screen.1 as f32), *text_color);

        // 返回绘制完成的图像
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_name() {
        let font = FontArc::try_from_slice(include_bytes!("./MiSans-Demibold.ttf")).unwrap();
        let _video = VideoConfig {
            screen: (1920, 1080),
            fps: 30,
            transition_sec: 2,
            output_path: PathBuf::from("./output.mp4"),
        };
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
            duration_sec: 5,
            background: None,
        };
        let _ = reader_frame(slides).unwrap();
    }
}
