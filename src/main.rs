use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader};

use plotters::prelude::*;
use std::path::Path;

const LOG_DIRECTORY: &str = "lotuslog";
const LOG_FILE_EXTENSION: &str = "log";
const IMG_EXTENSION: &str = "png";

fn main() {
    let dir = read_dir(LOG_DIRECTORY).unwrap();

    for entry in dir {
        let entry = entry.unwrap();
        let mut path = entry.path();
        if path.extension().unwrap() == LOG_FILE_EXTENSION {
            let data = parse_lotus_log(&path);
            path.set_extension(IMG_EXTENSION);

            generate_png(&data, path);
        }
    }
}

// 2021-12-07T10:48:27.857+0800    INFO    ffiwrapper      ffiwrapper/sealer_cgo.go:643    ZR: ffi C2 end: {"sector": "38764", "elapsed": 985.142026852}
fn parse_lotus_log<P: AsRef<Path>>(path: P) -> Vec<u32> {
    let file = File::open(&path).unwrap();
    let lines = BufReader::new(file).lines();

    let mut data = vec![];
    for line in lines {
        let line = line.unwrap();
        if line.contains("ffi C2 end") && line.contains("elapsed") {
            let v = line.split(": ");
            let elapsed = v
                .last()
                .unwrap()
                .to_string()
                .replace("}", "")
                .parse::<f64>()
                .unwrap() as u32;

            data.push(elapsed);
        }
    }

    data
}

fn generate_png<P: AsRef<Path>>(data: &Vec<u32>, img_path: P) {
    let root_area = BitMapBackend::new(&img_path, (1024, 786)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let max = data.iter().max().unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("bench memery usage", ("黑体", 40))
        .build_cartesian_2d(0..(data.len() as f64 * 1.1) as u32, 0.0..*max as f64 * 1.2)
        .unwrap();

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .light_line_style(&TRANSPARENT)
        .disable_x_mesh()
        .draw()
        .unwrap();

    // draw line
    chart
        .draw_series(LineSeries::new(
            data.iter()
                .enumerate()
                .map(|(idx, used)| (idx as u32, (*used) as f64)),
            &BLUE,
        ))
        .unwrap();
}
