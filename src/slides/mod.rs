pub mod slide;

use crate::{
    err_new, err_new_image, err_new_io, err_new_tryfrom,
    error::{Kind, Result},
    prelude::debug_print,
};
use image::{DynamicImage, GenericImage, GenericImageView};
use serde::{Deserialize, Serialize};
use slide::{render_frame, Slide};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoConfig {
    screen: (u32, u32),
    fps: u32,
    transition_sec: u32,
    work_dir: PathBuf,
    overlap: u32,
    step: u32,
    back_color: String,
    cover_time: u32,
    ending_time: u32,
    video_swip_speed: u32,
    width_slides: u32,
}

pub fn read_config(path: PathBuf) -> Result<VideoConfig> {
    let file = fs::read(path).map_err(|e| err_new_io!(e))?;
    let config: VideoConfig =
        serde_json::from_slice(&file).map_err(|e| err_new!(Kind::Other, &e.to_string()))?;
    Ok(config)
}

/// 组合所有图像块并生成最终视频。
///
/// # Parameters
/// - `save_name`: 最终视频文件名。
///
/// # Errors
/// - 如果图像处理或保存过程中发生错误，则返回 `Err`。
/// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
///
pub fn run<P: AsRef<Path>>(slides: &[Slide], config: &VideoConfig, save_name: P) -> Result<()> {
    let slidess = divide(slides, config);
    let mut results = Vec::with_capacity(slidess.len() + 2);

    for (index, &slides) in slidess.iter().enumerate() {
        let target = combain_slides(config, slides)?;
        if index == 0 {
            let cover = target.crop_imm(0, 0, config.screen.0, config.screen.1);
            let cover_pic_name = Path::new("cover.png");
            // 保存组合后的图像
            cover
                .save(config.work_dir.join(cover_pic_name))
                .map_err(|e| err_new_image!(e))?;
            debug_print(format!("{cover_pic_name:?} successed"));

            let cover_video_name = cover_pic_name.with_extension("mp4");
            generate_endpoint_video(config, cover_pic_name, &cover_video_name, config.cover_time)?;
            results.push(cover_video_name);
        }

        // 保存组合后的图像
        let mid_pic_name = format!("{index:0>2}.png");
        let mid_pic_name = Path::new(&mid_pic_name);
        target
            .save(config.work_dir.join(mid_pic_name))
            .map_err(|e| err_new_image!(e))?;
        debug_print(format!("{mid_pic_name:?} successed"));

        let mid_video_name = mid_pic_name.with_extension("mp4");
        generate_mid_video(config, slides.len() as u32, mid_pic_name, &mid_video_name)?;
        results.push(mid_video_name);

        if index == slidess.len() - 1 {
            let w = target.dimensions().0;
            let ending = target.crop_imm(w - config.screen.0, 0, config.screen.0, config.screen.1);
            let ending_pic_name = Path::new("ending.png");
            // 保存组合后的图像
            ending
                .save(config.work_dir.join(ending_pic_name))
                .map_err(|e| err_new_image!(e))?;
            debug_print(format!("{ending_pic_name:?} successed"));

            let ending_video_name = ending_pic_name.with_extension("mp4");
            generate_endpoint_video(
                config,
                ending_pic_name,
                &ending_video_name,
                config.ending_time,
            )?;
            results.push(ending_video_name);
        }
    }

    combain(config, &mut results, save_name.as_ref())?;
    Ok(())
}

/// 将图像块分割成多个子块。
///
/// # Results
/// 返回一个包含分割后子块的向量。
///
fn divide<'a>(slides: &'a [Slide], config: &VideoConfig) -> Vec<&'a [Slide]> {
    let len = slides.len();
    (0..len - config.overlap as usize)
        .step_by((config.step - config.overlap) as usize)
        .map(|i| &slides[i..(i + config.step as usize).min(len)])
        .collect()
}

/// 将多个图像块组合成一个完整的图像。
///
/// # Parameters
/// - `slides`: 要组合的图像块切片。
///
/// # Results
/// 如果成功，则返回组合后的 `DynamicImage`；如果失败，则返回 `Err`。
///
/// # Errors
/// - 如果 `slides` 为空，则返回 `Err`。
/// - 如果图像处理过程中发生错误，则返回 `Err`。
///
fn combain_slides(config: &VideoConfig, slides: &[Slide]) -> Result<DynamicImage> {
    if slides.is_empty() {
        return Err(err_new!(Kind::Other, "Empty slides"));
    }

    let len = u32::try_from(slides.len()).map_err(|e| err_new_tryfrom!(e))?;
    let mut target = DynamicImage::new_rgba8(len * config.width_slides, config.screen.1);

    // 将每张图片绘制到目标图像中
    for (i, item) in slides.iter().enumerate() {
        let img = render_frame(item).map_err(|e| err_new_image!(e))?;
        target
            .copy_from(&img, u32::try_from(i)? * config.width_slides, 0)
            .map_err(|e| err_new_image!(e))?;
    }
    Ok(target)
}

/// 生成视频封面或结尾视频。
///
/// # Parameters
/// - `pic_name`: 素材图片名称。
/// - `video_name`: 生成视频名称。
/// - `video_time`: 视频时长（秒）。
///
/// # Errors
/// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
///
fn generate_endpoint_video(
    config: &VideoConfig,
    pic_name: &Path,
    video_name: &Path,
    video_time: u32,
) -> Result<()> {
    ffmpeg(
        config,
        &[
            "-r",
            "1",
            "-loop",
            "1",
            "-i",
            pic_name.to_str().unwrap(),
            "-filter_complex",
            &format!(
                "color={}:s={}x{}:r={}[bg];[bg][0]overlay=shortest=1",
                config.back_color, config.screen.0, config.screen.1, config.fps
            ),
            "-preset",
            "fast",
            "-t",
            &video_time.to_string(),
            "-y",
            video_name.to_str().unwrap(),
        ],
    )?;
    debug_print(format!("{video_name:?} successed"));
    Ok(())
}

/// 生成中间部分的视频。
///
/// # Parameters
/// - `len`: 素材图片中 `slides` 数量。
/// - `pic_name`: 素材图片名称。
/// - `video_name`: 生成视频名称。
///
/// # Errors
/// - 如果 `FFmpeg` 命令执行失败，则返回 `Err`。
///
fn generate_mid_video(
    config: &VideoConfig,
    len: u32,
    pic_name: &Path,
    video_name: &Path,
) -> Result<()> {
    let adjust_len = len - config.overlap;
    let run_seconds = config.video_swip_speed * adjust_len + 1;
    let speed = config.width_slides / config.video_swip_speed;

    ffmpeg(
        config,
        &[
            "-r",
            "1",
            "-loop",
            "1",
            "-t",
            &run_seconds.to_string(),
            "-i",
            pic_name.to_str().unwrap(),
            "-filter_complex",
            &format!(
                "color={}:s={}x{}:r={}[bg];[bg][0]overlay=x=-t*{speed}:shortest=1",
                config.back_color, config.screen.0, config.screen.1, config.fps
            ),
            "-preset",
            "fast",
            "-y",
            video_name.to_str().unwrap(),
        ],
    )?;
    debug_print(format!("{video_name:?} successed"));
    Ok(())
}

#[allow(unused)]
/// 执行带有指定参数的FFmpeg命令
///
/// # Parameters
/// - `config: &VideoConfig` - 包含工作路径配置的结构体实例引用
/// - `args` - 传递给ffmpeg命令行工具的字符串参数切片
///
/// # Results
/// - 成功时返回Ok(())，失败时返回包含上下文信息的Err
///
/// # Errors
/// - 无法执行ffmpeg命令时返回IO错误
/// - ffmpeg进程返回非零状态码时打印stderr到控制台并返回Other类型错误
///
fn ffmpeg(config: &VideoConfig, args: &[&str]) -> Result<()> {
    let command = Command::new("ffmpeg")
        .current_dir(&config.work_dir)
        .args(args)
        .output()?;
    if !command.status.success() {
        println!("{}", String::from_utf8(command.stderr).unwrap());
        return Err(err_new!(Kind::Other, "FFmpeg command failed"));
    }
    Ok(())
}

/// 合并多个文件为单个输出文件，使用ffmpeg的concat协议
///
/// # Parameters
/// - `results`: 需要合并的源文件路径列表
/// - `save_name`: 合并后的输出文件路径
///
/// # Errors
/// - 如果文件写入或 `FFmpeg` 命令执行失败，则返回 `Err`。
///
fn combain(config: &VideoConfig, results: &mut [PathBuf], save_name: &Path) -> Result<()> {
    // 构建ffmpeg concat协议要求的输入文件列表字符串
    // 格式示例：file '/path/to/file1'\nfile '/path/to/file2'
    let result_str =
        results
            .iter()
            .fold(String::with_capacity(results.len() * 20), |mut init, s| {
                init.push_str(&format!("file {}\n", s.to_string_lossy()));
                init
            });

    // 将文件列表写入临时文本文件
    let list_file = config.work_dir.join("list.txt");
    std::fs::write(&list_file, result_str)?;

    // 调用ffmpeg执行合并操作参数说明：
    // -f concat 指定concat分离器
    // -i 输入文件列表
    // -c copy 使用流拷贝模式（不重新编码）
    // -y 覆盖输出文件
    ffmpeg(
        config,
        &[
            "-f",
            "concat",
            "-i",
            &list_file.to_string_lossy(),
            "-c",
            "copy",
            "-y",
            &save_name.to_string_lossy(),
        ],
    )?;

    println!("{} successed", save_name.to_string_lossy());

    // 清理临时文件（包含两个步骤）：
    // 1. 删除文件列表
    // 2. 删除所有中间结果文件及其对应的png文件
    let _ = std::fs::remove_file(&list_file);
    for result in results {
        let _ = std::fs::remove_file(config.work_dir.join(&result));
        result.set_extension("png");
        let _ = std::fs::remove_file(config.work_dir.join(result));
    }
    println!("cleanup successed");
    Ok(())
}
#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use super::*;

    #[test]
    fn test_read_config_valid() {
        let dir = PathBuf::from("./");
        let config_path = dir.join("config.json");
        let config_content = r#"
    {
      "screen": [1920, 1080],
      "fps": 30,
      "transition_sec": 2,
      "work_dir": ".",
      "overlap": 1,
      "step": 5,
      "back_color": "black",
      "cover_time": 3,
      "ending_time": 3,
      "video_swip_speed": 10,
      "width_slides": 192
    }
    "#;

        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let config = read_config(config_path).unwrap();
        assert_eq!(config.screen, (1920, 1080));
        assert_eq!(config.fps, 30);
        assert_eq!(config.transition_sec, 2);
    }

    #[test]
    fn test_read_config_invalid() {
        let dir = PathBuf::from("./");
        let config_path = dir.join("config.json");
        let config_content = r#"
    {
      "screen": [1920, 1080],
      "fps": "invalid_fps"
    }
    "#;

        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let result = read_config(config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_divide() {
        let slides = vec![Slide::default(); 10];
        let config = VideoConfig {
            screen: (1920, 1080),
            fps: 30,
            transition_sec: 2,
            work_dir: PathBuf::from("."),
            overlap: 1,
            step: 3,
            back_color: "black".to_string(),
            cover_time: 3,
            ending_time: 3,
            video_swip_speed: 10,
            width_slides: 192,
        };

        let divided = divide(&slides, &config);
        assert_eq!(divided.len(), 5);
        assert_eq!(divided[0].len(), 3);
        assert_eq!(divided[1].len(), 3);
        assert_eq!(divided[2].len(), 3);
        assert_eq!(divided[3].len(), 3);
        assert_eq!(divided[4].len(), 2);
    }

    #[test]
    fn test_combain_slides_empty() {
        let config = VideoConfig {
            screen: (1920, 1080),
            fps: 30,
            transition_sec: 2,
            work_dir: PathBuf::from("."),
            overlap: 1,
            step: 3,
            back_color: "black".to_string(),
            cover_time: 3,
            ending_time: 3,
            video_swip_speed: 10,
            width_slides: 192,
        };

        let result = combain_slides(&config, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ffmpeg_failure() {
        let config = VideoConfig {
            screen: (1920, 1080),
            fps: 30,
            transition_sec: 2,
            work_dir: PathBuf::from("."),
            overlap: 1,
            step: 3,
            back_color: "black".to_string(),
            cover_time: 3,
            ending_time: 3,
            video_swip_speed: 10,
            width_slides: 192,
        };

        let result = ffmpeg(&config, &["-invalid_flag"]);
        assert!(result.is_err());
    }
}
