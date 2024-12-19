mod disk;

use disk::{Disk, FATItem, BLOCK_SIZE, BLOCK_COUNT, EOF_BYTE};

use ansi_rgb::Foreground;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::Directory => write!(f, "Directory"),
            FileType::File => write!(f, "File") // 将字符串写入输出流f
        }
    }
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

impl Directory {
    fn new(name: &str) -> Directory {
        Directory {
            name: String::from(name),
            files: Vec::new()
        }
    }

    fn get_fcb(&self, name: &str) -> Option<(usize, &Fcb)> {
        let mut result = None;
        for i in 0..self.files.len() {
            if self.files[i].name.as_str() == name {
                result = Some((i, &self.files[i])); // 返回索引+对应FCB
                break;
            }
        }
        result
    }
}

impl fmt::Display for Directory {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Directory: '{}' Files", self.name)?;
        for file in &self.files {
            writeln!(
                f,
                "{}\t\t{}\t\tLength: {}",
                file.name, file.file_type, file.length
            )?;
        }
        fmt::Result::Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DiskOperator {
    pub disk: Disk,
    pub cur_dir: Directory,
}

impl DiskOperator {
    pub fn new(root_dir: Option<Directory>) -> DiskOperator { //初始化新磁盘
        print_info();
        println!("Creating a new disk...");

        let mut disk = Disk::new();

        let dir_data = bincode::serialize(&root_dir).unwrap();
        disk.insert_data_in_offset(dir_data.as_slice(), 0); // 将根目录序列化后写入磁盘
        disk.fat[0] = FATItem::EOF; // 根目录的FAT表项为EOF

        DiskOperator {
            disk,
            cur_dir: match root_dir {
                Some(directory) => directory,
                None => Directory {
                    name: String::from("root"),
                    files: vec![
                        Fcb {
                            name: String::from("."),
                            file_type: FileType::Directory,
                            first_cluster: 0,
                            length: 0
                        },
                        Fcb {
                            name: String::from(".."),
                            file_type: FileType::Directory,
                            first_cluster: 0,
                            length: 0
                        }
                    ]
                }
            }
        }

    }

    // 找到第一个unused
    pub fn find_empty_block(&self) -> Option<usize> {
        for i in 0..self.disk.fat.len() {
            if let FATItem::UnUsed = self.disk.fat[i] {
            }
        }
        None  
    }

    // 分配指定数量的块，返回块号数组
    pub fn allocate_block(&mut self, cnumber: usize) -> Result<Vec<usize>, &'static str> {
        print_info();
        println!("Allocating {} clusters...", cnumber);

        let mut clusters: Vec<usize> = Vec::with_capacity(cnumber);
        for i in 0..cnumber {
            clusters.push(match self.find_empty_block() {
                Some(cluster) =>cluster,
                _ => return Err("No enough space!"),
            });
            
            print_debug();
            println!("Allocated cluster: {}", clusters[i]);

            let cur_cluster = clusters[i];
            if i > 0 {
                self.disk.fat[clusters[i-1]] = FATItem::Cluster(cur_cluster);
            }
            self.disk.fat[cur_cluster] = FATItem::EOF;
        }
        Ok(clusters)
    }
    
}