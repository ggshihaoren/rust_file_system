mod disk;

use disk::{Disk, FATItem, BLOCK_SIZE, BLOCK_COUNT, EOF_BYTE};

use ansi_rgb::Foreground;
use serde::{Deserialize, Serialize};

pub fn print_info() {
    print!("{}", "[INFO]\t".fg(ansi_rgb::cyan_blue()));
}

pub fn print_debug() {
    print!("{}", "[DEBUG]\t".fg(ansi_rgb::magenta()));
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    File,
    Directory
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fcb {
    name: String, 
    file_type: FileType,
    first_cluster: usize, // 起始块号
    length: usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    name: String,
    files: Vec<Fcb>
}

#[derive(Serialize, Deserialize)]
pub struct DiskOperator {
    pub disk: Disk,
    pub cur_dir: Directory,
}