use std::{
    io::Write,
    str::Bytes,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::Engine;
use opencv::{
    core::{Mat, MatTraitConst, MatTraitConstManual},
    imgproc,
    videoio::{VideoCaptureTrait, VideoCaptureTraitConst},
};
use rand::{Rng, SeedableRng, TryRngCore};
use rand_chacha::ChaCha20Rng;
use rand_core::OsRng;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{
    AppState,
    error::{self, AppError, ErrorResponse},
    error_response,
};

// TODO: FIX THIS DESCRIPTION. create a struct for the return type when 200

#[derive(Serialize, ToSchema)]
struct HashResponse {
    hash: String,
    frame_basic: String,
    frame_grayscale: String,
    frame_downscale: String,
}

#[utoipa::path(
    post,
    description = "Получить хеш и фреймы лава лампы, использованные для генерации хеша",
    path = "/api/internal/hash",
    responses(
        (status = 200, description = "Хеш успешно создался", body = HashResponse),
        (status = 400, description = "Кадр лампы еще не готов (обычно такое только сразу после запуска сервиса)", body = ErrorResponse),
        (status = 500, description = "Что-то произошло", body = ErrorResponse),
    )
)]
pub async fn generate_hash(
    State(mut state): axum::extract::State<AppState>,
) -> Result<Response, AppError> {
    let frame = state.last_frame.lock().unwrap().clone();
    if frame.empty() {
        return Ok(error_response!(
            StatusCode::BAD_REQUEST,
            error::ErrorTypes::BadData,
            "Frame isn't ready yet, try again later"
        )
        .into_response());
    }

    let mut grayframe = Mat::default();
    imgproc::cvt_color(
        &frame,
        &mut grayframe,
        imgproc::COLOR_BGR2GRAY,
        0,
        // opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT, // ubuntu is fucking dogshit ,  comment out when build dockerfile
    )?;

    let mut downframe = Mat::default();
    let new_size = opencv::core::Size::new(64, 64);
    imgproc::resize(
        &grayframe,
        &mut downframe,
        new_size,
        0.0,
        0.0,
        imgproc::INTER_AREA,
    )?;

    if !downframe.is_continuous() {
        return Ok(error_response!(
            StatusCode::INTERNAL_SERVER_ERROR,
            error::ErrorTypes::BadData,
            "Video must be continious"
        ));
    }
    let frame_bytes = downframe.data_bytes()?;

    let mut os_rand = [0u8; 32];
    OsRng.try_fill_bytes(&mut os_rand)?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let ts_bytes = ts.to_be_bytes();

    let mut hasher = Sha256::new();
    hasher.update(frame_bytes);
    hasher.update(os_rand);
    hasher.update(ts_bytes);
    hasher.update(state.prev_hash);
    let result = hasher.finalize();
    let bytes: [u8; 32] = result.into();
    state.prev_hash = bytes;
    let result_hex = hex::encode(result);

    // let mut rng = ChaCha20Rng::from_seed(result.into());
    let response = HashResponse {
        hash: result_hex,
        frame_basic: mat_to_base64(&frame)?,
        frame_grayscale: mat_to_base64(&grayframe)?,
        frame_downscale: mat_to_base64(&downframe)?,
    };
    Ok((StatusCode::OK, axum::Json(response)).into_response())
}

fn mat_to_base64(mat: &opencv::core::Mat) -> anyhow::Result<String> {
    use opencv::core::Vector;

    if mat.empty() {
        return Err(anyhow::anyhow!("Frame is empty"));
    }

    let mut params: Vector<i32> = Vector::new();
    params.reserve(2);
    params.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    params.push(100);

    let mut buf: opencv::core::Vector<u8> = opencv::core::Vector::new();
    let success = opencv::imgcodecs::imencode(".jpg", mat, &mut buf, &params)?;
    if success {
        let imaged = base64::prelude::BASE64_STANDARD.encode(buf);
        return Ok(imaged);
    }

    Err(anyhow::anyhow!("Could not convert to .jpg"))
}
