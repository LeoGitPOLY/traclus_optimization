use crate::cluster::clustered_trajectory_store::ClusteredTrajStore;
use crate::spatial::geometry::Corridor;
use crate::spatial::geometry::Point;
use crate::spatial::input_od_line::InputODLine;
use crate::spatial::raw_trajectory_store::RawTrajStore;
use crate::spatial::trajectory::Trajectory;
use crate::utils_io::traclus_args::TraclusArgs;

use std::fs;
use std::io;
use std::path::Path;

use std::fs::File;
use std::io::{BufWriter, Write};

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

#[inline]
fn parse_line_to_od(line: &str, index: usize) -> io::Result<InputODLine> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 6 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Line {} is malformed: {}", index + 1, line),
        ));
    }

    Ok(InputODLine {
        line_id: parts[0]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse line_id"))?,
        weight: parts[1]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse weight"))?,
        start: Point {
            x: parts[2].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Failed to parse start x")
            })?,
            y: parts[3].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Failed to parse start y")
            })?,
        },
        end: Point {
            x: parts[4]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse end x"))?,
            y: parts[5]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse end y"))?,
        },
    })
}

pub fn parse_input_data(args: &TraclusArgs) -> RawTrajStore {
    let content: String = read_file(&args.infile).expect("Failed to read input file");
    let mut trajectory_storage: RawTrajStore = RawTrajStore::new(args.max_angle);

    for (index, line) in content.lines().enumerate() {
        let od_line: InputODLine = parse_line_to_od(line, index).unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        });

        let trajectory: Trajectory = Trajectory::new(od_line, args.segment_size);
        trajectory_storage.add_trajectory(trajectory);
    }

    trajectory_storage
}

/// Writes corridor data to a text file in the specified format.
///
/// Output format:
/// - Header: "name\tweight\tcoordinates"
/// - Data: "{id}\t{weight}\tLINESTRING({x1} {y1}, {x2} {y2})"
///
/// Filename format:
/// {input_basename}.{max_dist}.{min_density}.{max_angle}.{segment_size}.corridorlist
///
/// Panics if file writing fails, as this indicates a critical I/O error.
pub fn parse_output_data(args: &TraclusArgs, clust_storage: &ClusteredTrajStore) {
    let output_filename = build_output_filename(args);

    // Create file with buffered writer for better performance
    let file = File::create(&output_filename).expect("Failed to create output file");

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "name\tweight\tcoordinates").expect("Failed to write header");

    // Write each corridor
    for corridor in &clust_storage.corridors {
        write_corridor(&mut writer, corridor);
    }

    // Flush to ensure all data is written
    writer.flush().expect("Failed to flush output file");

    println!("Output written to: {}", output_filename);
}

/// Constructs the output filename from input file and clustering parameters.
///
/// Format: {basename}.{max_dist}.{min_density}.{max_angle}.{segment_size}.corridorlist
fn build_output_filename(args: &TraclusArgs) -> String {
    let input_path = Path::new(&args.infile);
    let basename = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    format!(
        "{}.{}.{}.{}.{}.corridorlist",
        basename, args.max_dist, args.min_density, args.max_angle, args.segment_size
    )
}

/// Writes a single corridor to the output file in LINESTRING format.
///
/// Format: {id}\t{weight}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_corridor(writer: &mut BufWriter<File>, corridor: &Corridor) {
    writeln!(
        writer,
        "{}\t{}\tLINESTRING({} {}, {} {})",
        corridor.id,
        corridor.weight,
        corridor.start.x,
        corridor.start.y,
        corridor.end.x,
        corridor.end.y
    )
    .expect("Failed to write corridor");
}
