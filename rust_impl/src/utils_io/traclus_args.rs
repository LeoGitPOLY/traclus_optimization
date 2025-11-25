use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Traclus DL Optimized in Rust")]
pub struct TraclusArgs {
    #[arg(short = 'i', long)]
    pub infile: String,

    #[arg(
        short = 'd',
        long = "max_dist",
        default_value_t = 250.0,
        value_parser = |v: &str| {
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < 0.0 {
                Err(String::from("max_dist must be >= 0"))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_dist: f64, // RANGE VALUE IS [0; INFINITY[

    #[arg(
        short = 'n',
        long = "min_density",
        default_value_t = 2,
        value_parser = |v: &str| {
            let val: u32 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < 1 {
                Err(String::from("min_density must be >= 1"))
            } else {
                Ok(val)
            }
        }
    )]
    pub min_density: u32, // RANGE VALUE IS [1; INFINITY[

    #[arg(
        short = 'a',
        long = "max_angle",
        default_value_t = 5.0,
        value_parser = |v: &str| {
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < 0.0 || val > 220.5 {
                Err(String::from("max_angle must be in range 0.0..22.5"))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_angle: f64, // RANGE VALUE IS [0; 22.5]

    #[arg(
        short = 's',
        long = "segment_size",
        default_value_t = 500.0,
        value_parser = |v: &str| {
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val <= 0.0 {
                Err(String::from("segment_size must be > 0"))
            } else {
                Ok(val)
            }
        }
    )]
    pub segment_size: f64, // RANGE VALUE IS ]0; INFINITY[
}
