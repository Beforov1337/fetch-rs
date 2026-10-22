use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "fetch", version, about = "3D ASCII distro fetch tool in Rust")]
pub struct Cli {
    /// Use a logo from fastfetch by name (or gentoo default)
    #[arg(short, long)]
    pub logo: Option<String>,

    /// Lock rotation to X axis only
    #[arg(long)]
    pub rotate_x: bool,

    /// Lock rotation to Y axis only
    #[arg(long)]
    pub rotate_y: bool,

    /// Speed multiplier
    #[arg(short, long, default_value_t = 1.0)]
    pub speed: f32,

    /// Scale the logo
    #[arg(long, default_value_t = 1.0)]
    pub size: f32,

    /// Scale the 3D depth
    #[arg(long, default_value_t = 1.0)]
    pub depth: f32,

    /// Override render height in rows
    #[arg(long)]
    pub height: Option<usize>,

    /// Just the logo, no system info
    #[arg(long)]
    pub no_info: bool,

    /// Disable logo coloring
    #[arg(long)]
    pub no_color: bool,

    /// Stop after n frames (0 for infinite)
    #[arg(long, default_value_t = 2000)]
    pub frames: usize,

    /// Run forever until keypress or Ctrl-C
    #[arg(long)]
    pub infinite: bool,

    /// Shading ramp mode: ascii, blocks, or sextants
    #[arg(long, default_value = "ascii")]
    pub shading_mode: String,

    /// Custom shading ramp characters
    #[arg(long)]
    pub shading_chars: Option<String>,

    /// Draw a border box around the info block
    #[arg(long)]
    pub box_border: bool,
}
