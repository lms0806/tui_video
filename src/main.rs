use std::{sync::Arc, time::Duration};

use rusty_ytdl::*;
use tokio::{
    join,
    sync::mpsc::{self, Receiver, Sender, error::SendError},
    time::{Instant, sleep},
};

use std::io::stdin;

use crate::{ascii::rgb_to_colored_ascii, ffmpeg_fn::VideoProcessor};

mod ascii;
mod ffmpeg_fn;
mod tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut url = String::new();
    println!("input youtube url");
    stdin().read_line(&mut url).unwrap();
    let (sender, receiver) = mpsc::channel(32);

    let info = Video::new(&url)?.get_info().await?;
    let format = info
        .formats
        .iter()
        .filter(|f| f.has_video && f.mime_type.container == "webm")
        .min_by_key(|f| f.bitrate)
        .expect("No WebM format found");

    let target_itag = format.itag;

    let video_option = VideoOptions {
        quality: rusty_ytdl::VideoQuality::LowestVideo,
        filter: VideoSearchOptions::Custom(Arc::new(move |f| f.itag == target_itag)),
        ..Default::default()
    };
    let video = Video::new_with_options(url, video_option).unwrap();

    // let res = video.download(std::path::Path::new(r"test.mp4")).await;

    // if let Err(e) = res {
    //     println!("{:?}", e);
    // }

    join!(get_video(video, sender), print_tui(receiver));

    println!("END");
    Ok(())
}

async fn get_video(
    video: Video<'_>,
    sen: Sender<bytes::Bytes>,
) -> Result<(), SendError<bytes::Bytes>> {
    let stream = video.stream().await.unwrap();

    while let Some(chunk) = stream.chunk().await.unwrap() {
        sen.send(chunk).await?;
    }

    Ok(())
}

async fn print_tui(rec: Receiver<bytes::Bytes>) -> anyhow::Result<()> {
    let mut terminal = tui::init()?;

    // 1. 프레임 데이터(Vec<u8>)를 받을 채널
    let (frame_tx, mut frame_rx) = mpsc::channel(10); // 버퍼를 작게 잡아 지연을 줄임

    // 2. 디코더 스레드 실행 (VideoProcessor가 저쪽에서 돕니다)
    let processor = VideoProcessor::new(rec)?;
    ffmpeg_fn::spawn_decoding_thread(processor, frame_tx);

    let mut tui_frame = Vec::new();

    // FPS 설정 (30 FPS 기준)
    let target_frame_duration = Duration::from_millis(33);

    // 3. 메인 루프: 받아서 -> 변환하고 -> 그리고 -> 쉰다
    while let Some(frame_data) = frame_rx.recv().await {
        let start_time = Instant::now();

        // (1) ASCII 변환
        // frame_data는 이미 100x50 RGB로 리사이징되어 옴 (VideoProcessor에서 처리함)
        tui_frame.clear();
        rgb_to_colored_ascii(&frame_data, 100, 50, &mut tui_frame);

        // (2) 그리기
        tui::draw_frame(&mut terminal, &tui_frame)?;

        // (3) FPS 시간 맞추기
        let elapsed = start_time.elapsed();
        if elapsed < target_frame_duration {
            sleep(target_frame_duration - elapsed).await;
        }
    }

    tui::restore()?;
    Ok(())
}
