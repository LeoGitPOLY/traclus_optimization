use crate::clustering::cluster_member::ClusterMember;
use crate::clustering::corridor::Corridor;
use crate::io::args::TraclusArgs;
use crate::storage::clustered_trajectories::ClusteredTrajectories;
use std::path::Path;

use std::fs::File;
use std::io::{BufWriter, Write};

pub enum SegmentOutputFormat {
    OldTraclus,
    NewTraclus,
}

// Generate the corridor output file to a text file
pub fn generate_corridor_file(args: &TraclusArgs, clust_storage: &ClusteredTrajectories) {
    let output_filename: String = build_corridor_output_filename(args);

    let file: File = File::create(&output_filename).expect("Failed to create corridor output file");
    let mut writer: BufWriter<File> = BufWriter::new(file);

    writeln!(writer, "name\tweight\tcoordinates").expect("Failed to write corridor header");

    for corridor in &clust_storage.corridors {
        write_single_corridor(&mut writer, corridor);
    }

    writer.flush().expect("Failed to flush corridor file");
    println!("Corridor output written to: {}", output_filename);
}

// Generate the segment output file to a text file
pub fn generate_segment_file(
    args: &TraclusArgs,
    clust_storage: &ClusteredTrajectories,
    format: SegmentOutputFormat,
) {
    let output_filename: String = build_segment_output_filename(args, &format);

    let file: File = File::create(&output_filename).expect("Failed to create segment output file");
    let mut writer: BufWriter<File> = BufWriter::new(file);

    write_segment_header(&mut writer, &format);

    for (corridor_id, cluster_member) in clust_storage.get_all_cluster_members_iter() {
        match format {
            SegmentOutputFormat::OldTraclus => {
                write_single_segment_old(&mut writer, corridor_id, cluster_member);
            }
            SegmentOutputFormat::NewTraclus => {
                write_single_segment_new(&mut writer, corridor_id, cluster_member);
            }
        }
    }

    writer.flush().expect("Failed to flush segment file");

    println!("Segment output written to: {}", output_filename);
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

fn build_segment_output_filename(args: &TraclusArgs, format: &SegmentOutputFormat) -> String {
    let input_path: &Path = Path::new(&args.file);
    let basename: &str = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let parent_dir: &Path = input_path.parent().unwrap_or_else(|| Path::new("."));

    let suffix = match format {
        SegmentOutputFormat::OldTraclus => "segmentlist_old",
        SegmentOutputFormat::NewTraclus => "segmentlist_new",
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
) {
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
    .expect("Failed to write new segment");
}

// Format: {trajectory_id}\t{weight}\t{angle}\t{corridor_id}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_single_segment_old(
    writer: &mut BufWriter<File>,
    corridor_id: i32,
    cluster_member: &ClusterMember,
) {
    let end_point = cluster_member.end_point();
    writeln!(
        writer,
        "{}\t{}\t{}\t{}\tLINESTRING({} {}, {} {})",
        cluster_member.traj_id,
        cluster_member.weight,
        cluster_member.angle(),
        corridor_id,
        cluster_member.start.x,
        cluster_member.start.y,
        end_point.x,
        end_point.y
    )
    .expect("Failed to write old segment");
}

// Format: {id}\t{weight}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_single_corridor(writer: &mut BufWriter<File>, corridor: &Corridor) {
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

// Writes the segment header based on the specified format.
// Old Traclus: id weight angle corridor_id coordinates
// New Traclus: corridor_id trajectory_id segment_id weight angle coordinates
fn write_segment_header(writer: &mut BufWriter<File>, format: &SegmentOutputFormat) {
    match format {
        SegmentOutputFormat::OldTraclus => {
            writeln!(writer, "id\tweight\tangle\tcorridor_id\tcoordinates")
                .expect("Failed to write old segment header");
        }
        SegmentOutputFormat::NewTraclus => {
            writeln!(
                writer,
                "corridor_id\ttrajectory_id\tsegment_id\tweight\tangle\tcoordinates"
            )
            .expect("Failed to write new segment header");
        }
    }
}
