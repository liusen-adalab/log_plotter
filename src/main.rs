use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use plotters::prelude::*;
use regex::Regex;
use std::path::Path;
#[macro_use]
extern crate lazy_static;
use chrono::{self, DateTime, TimeZone, Utc};

lazy_static! {
    // static ref MEM_LOG_PATH: &'static str = "sen_bench/bench-1th-sim5/mem1.log";
    static ref MEM_LOG_PATH: &'static str = "acc-log/mem.log";

    // static ref MEM_LOG_PATH: &'static str = "sen_bench/sim1-cpu-0-20/mem0.log";
    static ref BENCH_LOG_PATH: &'static str = "log/bench.log";
    static ref START_TIME_STAMP: usize = get_start_time_stamp(&*MEM_LOG_PATH);
    static ref MEM_DATA: Vec<u32> = parse_mem_log(&*MEM_LOG_PATH);
    static ref BENCH_DATA: (Vec<usize>, Vec<(usize, String)>) = parse_bench_log(&*BENCH_LOG_PATH);
}
// static PNG_PATH: &'static str = "sen_bench/sim1-cpu-0-20/mem0.png";
// static PNG_PATH: &'static str = "sen_bench/bench-1th-sim5/mem1.log";
static PNG_PATH: &'static str = "acc-log/mem.png";

// 2021-11-26 11:19:28         1032015        2901      920876          16      108237     1024719
fn get_start_time_stamp<P: AsRef<Path> + Display>(path: P) -> usize {
    let lines = read_bench_to_lines(path);
    let first_line = lines.skip(1).next().unwrap().unwrap();
    let time_str = first_line.split_whitespace().next().unwrap();

    Utc.datetime_from_str(time_str, "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .timestamp() as usize
}

fn parse_mem_log<P: AsRef<Path> + Display>(path: P) -> Vec<u32> {
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => panic!("couldn't open {}: {}", path, err),
    };

    let regex = Regex::new(
        r"^(?P<date>\d+-\d+-\d+)\s+(?P<time>\d+:\d+:\d+)\s+(?P<total>\d+)\s+(?P<used>\d+)",
    )
    .unwrap();
    let mut data = vec![];

    let lines = BufReader::new(file).lines();
    lines.skip(1).map(|line| line.unwrap()).for_each(|line| {
        let caps = regex.captures(&line).unwrap();

        let used: u32 = caps.name("used").unwrap().as_str().parse().unwrap();

        data.push(used);
    });

    data
}

// 2021-12-01T20:03:49.145 INFO bellperson > | 1th | e-l_s | process id :3882167 | Cpu Usage 0% | Physical MemUsage 66190MB
// 2021-12-01T20:03:49.145 INFO bellperson > | 1th | e-l_s |

fn parse_bench_log<P: AsRef<Path>>(path: P) -> (Vec<usize>, Vec<(usize, String)>) {
    let lines = read_bench_to_lines(path);
    let regex =
        Regex::new(r"^(?P<datetime>\d+-\d+-\d+T\d+:\d+:\d+\.\d+)\s+.*?\|\s*(?P<th>\d+)th\s*\|\s*(?P<stage>.*?)\s*\|")
            .unwrap();

    let mut bench_log_data = vec![];
    let mut texts = vec![];
    lines.map(|f| f.unwrap()).for_each(|line| {
        let caps = regex.captures(&line).unwrap();
        let mut datetime = caps.name("datetime").unwrap().as_str().to_string();
        datetime.push('Z');

        #[cfg(test)]
        {
            println!("{}", datetime.parse::<DateTime<Utc>>().unwrap());
        }

        let time_stamp = datetime.parse::<DateTime<Utc>>().unwrap().timestamp();
        let stage = caps.name("stage").unwrap().as_str().to_string();

        let th = caps
            .name("th")
            .unwrap()
            .as_str()
            .to_string()
            .parse::<usize>()
            .unwrap();

        bench_log_data.push(time_stamp as usize);
        texts.push((th, stage));
    });

    (bench_log_data, texts)
}

fn read_bench_to_lines<P: AsRef<Path>>(path: P) -> Lines<BufReader<File>> {
    let file = File::open(path).expect("failed to open file");
    let reader = BufReader::new(file);
    reader.lines()
}

fn main() {
    let root_area = BitMapBackend::new(PNG_PATH, (1024, 786)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("bench memery usage", ("黑体", 40))
        .build_cartesian_2d(0..1000usize, 0.0..1000.0)
        .unwrap();
    // configure mesh
    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .light_line_style(&TRANSPARENT)
        .disable_x_mesh()
        .draw()
        .unwrap();

    // let bench_data = parse_bench_log("bench-test.log");
    // let start_time_stamp = get_start_time_stamp("mem.log");

    // let iter = bench_data.0.iter().map(|time_stamp| {
    //     let index = time_stamp - start_time_stamp;
    //     (index, MEM_DATA[index] as f64 / 1000.0)
    // });

    // let plot_text = bench_data.1;
    // draw stage point
    // text: (th, stage)
    // chart
    //     .draw_series(PointSeries::of_element(
    //             iter,
    //         5,
    //         ShapeStyle::from(&RED).filled(),
    //         &|coord, size, style| {
    //             EmptyElement::at(coord)
    //                 + Circle::new((0, 0), size, style)
    //                 + Text::new(
    //                     format!("{:?}", plot_text[coord.0]),
    //                     (0, 15),
    //                     ("sans-serif", 15),
    //                 )
    //         },
    //     ))
    //     .unwrap();

    // draw line
    chart
        .draw_series(LineSeries::new(
            MEM_DATA
                .iter()
                .enumerate()
                .map(|(idx, used)| (idx, (*used) as f64 / 1000.0)),
            &BLUE,
        ))
        .unwrap();
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};

    use crate::parse_bench_log;

    #[test]
    fn test_add_points() {
        let input = "2021-12-01T20:03:49.145Z";
        let time_stamp = input.parse::<DateTime<Utc>>().unwrap();
        println!("{}", time_stamp);
    }

    #[test]
    fn test_parse_bench_log() {
        let path = "bench-test.log";
        let data = parse_bench_log(path);
        for d in data {
            println!("{}", d.0);
            println!("{}", d.1);
            println!("{}", d.2);
        }
    }
}
