use crate::clustering::objects::cluster_member::ClusterMember;
use crate::clustering::objects::corridor::Corridor;
use crate::clustering::storage::clustered_trajectories::ClusteredTrajectories;
use crate::gui::app_events::AppError;
use crate::gui::event_singleton::emit_error;
use crate::io::args::TraclusArgs;
use std::path::Path;

use std::fs::File;
use std::io::{self, BufWriter, Write};

pub enum SegOutFormat {
    OldTraclus,
    NewTraclus,
}

// Generate the corridor output file to a text file
pub fn generate_corridor_file(
    args: &TraclusArgs,
    clust_storage: &ClusteredTrajectories,
) -> Option<()> {
    let output_filename: String = build_corridor_output_filename(args);

    let file: File = match File::create(&output_filename) {
        Ok(f) => f,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to create corridor output file: {}",
                err
            )));
            return None;
        }
    };

    let mut writer: BufWriter<File> = BufWriter::new(file);

    if let Err(err) = writeln!(writer, "name\tweight\tcoordinates") {
        emit_error(AppError::IoError(format!(
            "Failed to write corridor header: {}",
            err
        )));
        return None;
    }

    for corridor in &clust_storage.corridors {
        if let Err(err) = write_single_corridor(&mut writer, corridor) {
            emit_error(AppError::IoError(format!(
                "Failed to write corridor: {}",
                err
            )));
            return None;
        }
    }

    if let Err(err) = writer.flush() {
        emit_error(AppError::IoError(format!(
            "Failed to flush corridor file: {}",
            err
        )));
        return None;
    }

    println!("Corridor output written to: {}", output_filename);

    Some(())
}

// Generate the segment output file to a text file
pub fn generate_segment_file(
    args: &TraclusArgs,
    clust_storage: &ClusteredTrajectories,
    format: SegOutFormat,
) -> Option<()> {
    let output_filename = build_segment_output_filename(args, &format);

    let file = match File::create(&output_filename) {
        Ok(f) => f,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to create segment output file: {}",
                err
            )));
            return None;
        }
    };

    let mut writer = BufWriter::new(file);

    if let Err(err) = write_segment_header(&mut writer, &format) {
        emit_error(AppError::IoError(format!(
            "Failed to write segment header: {}",
            err
        )));
        return None;
    }

    for (corridor_id, cluster_member) in clust_storage.get_all_cluster_members_iter() {
        let result: io::Result<()> = match format {
            SegOutFormat::OldTraclus => {
                write_single_segment_old(&mut writer, corridor_id, cluster_member)
            }
            SegOutFormat::NewTraclus => {
                write_single_segment_new(&mut writer, corridor_id, cluster_member)
            }
        };

        if let Err(err) = result {
            emit_error(AppError::IoError(format!(
                "Failed to write segment: {}",
                err
            )));
            return None;
        }
    }

    if let Err(err) = writer.flush() {
        emit_error(AppError::IoError(format!(
            "Failed to flush segment file: {}",
            err
        )));
        return None;
    }

    println!("Segment output written to: {}", output_filename);

    Some(())
}

fn build_corridor_output_filename(args: &TraclusArgs) -> String {
    let input_path: &Path = Path::new(&args.file);
    let basename: &str = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let parent_dir: &Path = input_path.parent().unwrap_or_else(|| Path::new("."));

    format!(
        "{}/{}[{}-{}-{}-{}-{}].corridorlist.txt",
        parent_dir.display(),
        basename,
        args.max_dist.round(),
        args.min_density,
        args.max_angle.round(),
        args.segment_size.round(),
        args.mode,
    )
}

fn build_segment_output_filename(args: &TraclusArgs, format: &SegOutFormat) -> String {
    let input_path: &Path = Path::new(&args.file);
    let basename: &str = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let parent_dir: &Path = input_path.parent().unwrap_or_else(|| Path::new("."));

    let suffix = match format {
        SegOutFormat::OldTraclus => "segmentlist_old",
        SegOutFormat::NewTraclus => "segmentlist",
    };

    format!(
        "{}/{}[{}-{}-{}-{}-{}].{}.txt",
        parent_dir.display(),
        basename,
        args.max_dist.round(),
        args.min_density,
        args.max_angle.round(),
        args.segment_size.round(),
        args.mode,
        suffix
    )
}

// Format: {corridor_id}\t{trajectory_id}\t{segment_id}\t{weight}\t{angle}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_single_segment_new(
    writer: &mut BufWriter<File>,
    corridor_id: i32,
    cluster_member: &ClusterMember,
) -> io::Result<()> {
    let end_point = cluster_member.end_point();
    writeln!(
        writer,
        "{}\t{}\t{}\t{}\t{}\tLINESTRING({} {}, {} {})",
        corridor_id,
        cluster_member.traj_id,
        cluster_member.segment_id,
        cluster_member.weight,
        cluster_member.angle(),
        cluster_member.start.x,
        cluster_member.start.y,
        end_point.x,
        end_point.y
    )
}

// Format: {trajectory_id:segment_id}\t{weight}\t{angle}\t{corridor_id}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_single_segment_old(
    writer: &mut BufWriter<File>,
    corridor_id: i32,
    cluster_member: &ClusterMember,
) -> io::Result<()> {
    let end_point = cluster_member.end_point();
    let start_str = cluster_member.start.x.to_string() + ":" + &cluster_member.start.y.to_string();
    let segment_id = cluster_member.traj_id.to_string() + ":" + &start_str;

    writeln!(
        writer,
        "{}\t{}\t{}\t{}\tLINESTRING({} {}, {} {})",
        segment_id,
        cluster_member.weight,
        cluster_member.angle(),
        corridor_id,
        cluster_member.start.x,
        cluster_member.start.y,
        end_point.x,
        end_point.y
    )
}

// Format: {id}\t{weight}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_single_corridor(writer: &mut BufWriter<File>, corridor: &Corridor) -> io::Result<()> {
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
}

// Writes the segment header based on the specified format.
// Old Traclus: id weight angle corridor_id coordinates
// New Traclus: corridor_id trajectory_id segment_id weight angle coordinates
fn write_segment_header(writer: &mut BufWriter<File>, format: &SegOutFormat) -> io::Result<()> {
    match format {
        SegOutFormat::OldTraclus => {
            writeln!(writer, "id\tweight\tangle\tcorridor_id\tcoordinates")
        }
        SegOutFormat::NewTraclus => {
            writeln!(
                writer,
                "corridor_id\ttrajectory_id\tsegment_id\tweight\tangle\tcoordinates"
            )
        }
    }
}
