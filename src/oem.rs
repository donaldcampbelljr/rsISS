use std::fmt;
use std::fs::File;
use std::io::copy;
use std::process::{ExitCode, Termination};
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use polars::df;
use polars::error::PolarsResult;
use polars::frame::DataFrame;
use polars::prelude::NamedFrom;
use tempfile::Builder;

pub const ISS_OEM_URL: &str = "https://nasa-public-data.s3.amazonaws.com/iss-coords/current/ISS_OEM/ISS.OEM_J2K_EPH.txt";

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Http(reqwest::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Http(e) => write!(f, "HTTP error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// ── Satellite ────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Satellite {
    pub trajectory_summary: String,
    pub meta_summary: String,
    pub coordinates: DataFrame,
    pub x_coord_vec: Vec<f64>,
    pub y_coord_vec: Vec<f64>,
    pub z_coord_vec: Vec<f64>,
}

impl Satellite {
    /// Constructs a new instance of [`Satellite`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl Termination for Satellite {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

// ── Functions ────────────────────────────────────────────────────────────────

#[tokio::main]
pub async fn download_file(url: &str) -> Result<String> {
    let tmp_dir = Builder::new().prefix("example").tempdir()?;

    let response = reqwest::get(url).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        let fname = tmp_dir.path().join(fname);
        File::create(fname)?
    };

    let content = response.text().await?;
    copy(&mut content.as_bytes(), &mut dest)?;

    Ok(content)
}

pub fn construct_oem(content: &String) -> Satellite {
    let mut sat = Satellite::new();

    let mut previous_token = "nothing";

    let mut meta_body_vec: Vec<String> = Vec::new();
    let mut traj_body_vec: Vec<String> = Vec::new();
    let mut count = 1;

    let mut count_vec: Vec<i32> = Vec::new();
    let mut date_time_vec: Vec<DateTime<FixedOffset>> = Vec::new();
    let mut x_coord_vec: Vec<f64> = Vec::new();
    let mut y_coord_vec: Vec<f64> = Vec::new();
    let mut z_coord_vec: Vec<f64> = Vec::new();

    let gmt_offset = FixedOffset::west_opt(0).unwrap();

    for line in content.lines().take(60) {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        let search_meta_start = "META_START";
        let search_meta_end = "META_END";
        let search_comment = "COMMENT";
        let search_comment_source = "Source";
        let search_comment_trajectory = "TRAJECTORY";
        let search_comment_end = "End";
        let coordinates_start = "Coordinates Start";

        if tokens.get(0).unwrap_or(&"").contains(search_meta_start) {
            previous_token = search_meta_start;
        }

        if tokens.get(0).unwrap_or(&"").contains(search_meta_end) {
            previous_token = search_meta_end;
        }

        if tokens.get(0).unwrap_or(&"").contains(search_comment) {
            if tokens.len() > 1 && tokens.get(1).unwrap_or(&"").contains(search_comment_source) {
                previous_token = search_comment_source;
            }

            if tokens.len() > 1
                && tokens
                    .get(1)
                    .unwrap_or(&"")
                    .contains(search_comment_trajectory)
            {
                previous_token = search_comment_trajectory;
            }

            if tokens.len() > 1 && tokens.get(1).unwrap_or(&"").contains(search_comment_end) {
                previous_token = search_comment_end;
            }
        }

        let breaking = "2022-02-18T12:00:00.000";

        if tokens.get(0).unwrap_or(&"").contains(breaking) {
            previous_token = breaking;
        }

        match previous_token {
            "TRAJECTORY" => {
                traj_body_vec.push(line.replace("COMMENT", ""));
            }
            "META_START" | "META_END" => {
                meta_body_vec.push(line.replace("COMMENT", ""));
            }
            "End" => {
                previous_token = coordinates_start;
            }
            "Coordinates Start" => {
                let time_stamp = *tokens.get(0).unwrap();
                let datetime =
                    NaiveDateTime::parse_from_str(time_stamp, "%Y-%m-%dT%H:%M:%S.%3f").unwrap();
                let gmt_datetime = gmt_offset.from_local_datetime(&datetime).unwrap();

                let x_coord = tokens
                    .get(1)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Failed to parse x coord as f64"));

                let y_coord = tokens
                    .get(2)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Failed to parse y coord as f64"));

                let z_coord = tokens
                    .get(3)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Failed to parse z coord as f64"));

                count_vec.push(count);
                date_time_vec.push(gmt_datetime);
                x_coord_vec.push(x_coord);
                y_coord_vec.push(y_coord);
                z_coord_vec.push(z_coord);

                count += 1;
            }
            _ => {}
        }
    }

    let coord_df: PolarsResult<DataFrame> = df!(
        "counts"         => &count_vec,
        "x coordinates"  => &x_coord_vec,
        "y coordinates"  => &y_coord_vec,
        "z coordinates"  => &z_coord_vec,
    );

    sat.coordinates = coord_df.unwrap();

    // Workaround: expose coord vecs directly since extracting Vec<f64> from DataFrame is awkward
    sat.x_coord_vec = x_coord_vec;
    sat.y_coord_vec = y_coord_vec;
    sat.z_coord_vec = z_coord_vec;

    sat.meta_summary = meta_body_vec.join("\n");
    sat.trajectory_summary = traj_body_vec.join("\n");

    sat
}
