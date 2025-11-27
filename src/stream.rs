use std::sync::{Arc, Mutex};

use ffmpeg_next::{self as ffmpeg, format::Pixel, frame::Video, software::scaling::Flags};
pub fn stream_loop(last_frame: Arc<Mutex<opencv::core::Mat>>) -> anyhow::Result<()> {
    tracing_subscriber::fmt().pretty().init();
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);

    let mut ictx = || -> ffmpeg::format::context::Input {
        loop {
            match ffmpeg::format::input(&std::env::var("RTMP_URL").unwrap()) {
                Ok(ictx) => return ictx,
                Err(why) => {
                    tracing::error!(
                        "Could not find a video stream, it is probably missing: {}",
                        why
                    );
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }();

    let vstream = ictx.streams().best(ffmpeg::media::Type::Video).unwrap();
    let vstream_idx = vstream.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(vstream.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    for (stream, packet) in ictx.packets() {
        if stream.index() == vstream_idx {
            decoder.send_packet(&packet)?;

            let mut dframe = Video::empty();
            while decoder.receive_frame(&mut dframe).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&dframe, &mut rgb_frame)?;

                let width = rgb_frame.width();
                let height = rgb_frame.height();
                let data = rgb_frame.data(0);
                let stride = rgb_frame.stride(0);

                let oframe = unsafe {
                    opencv::core::Mat::new_rows_cols_with_data_unsafe(
                        height as i32,
                        width as i32,
                        opencv::core::CV_8UC3,
                        data.as_ptr() as *mut core::ffi::c_void,
                        stride as usize,
                    )
                }?;

                *last_frame.lock().unwrap() = oframe;
                break;
            }
        }
    }

    decoder.send_eof()?;

    Ok(())
}
