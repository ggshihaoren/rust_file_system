use crate::disk::{Disk, FATItem, BLOCK_SIZE};

use ansi_rgb::Foreground;
use core::panic;
use serde::{Deserialize, Serialize};
use std::{fmt, string::String, usize, vec::Vec};

pub fn print_info() {
    print!("{}", "[INFO]\t".fg(ansi_rgb::cyan_blue()));
}

pub fn print_debug() {
    print!("{}", "[DEBUG]\t".fg(ansi_rgb::magenta()));
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    File,
    Directory,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::Directory => write!(f, "Directory"),
            FileType::File => write!(f, "File"), // 将字符串写入输出流f
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fcb {
    name: String,
    file_type: FileType,
    first_cluster: usize, // 起始块号
    length: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    name: String,
    files: Vec<Fcb>,
}

impl Directory {
    fn new(name: &str) -> Directory {
        Directory {
            name: String::from(name),
            files: Vec::new(),
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

    pub fn get_file_type(&self, name: &str) -> Option<FileType> {
        match self.get_fcb(name) {
            Some((_, fcb)) => Some(fcb.file_type.clone()),
            None => None,
        }
    }
}

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub fn new(root_dir: Option<Directory>) -> DiskOperator {
        //初始化新磁盘
        // print_info();
        // println!("Creating a new disk...");

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
                            length: 0,
                        },
                        Fcb {
                            name: String::from(".."),
                            file_type: FileType::Directory,
                            first_cluster: 0,
                            length: 0,
                        },
                    ],
                },
            },
        }
    }

    // 找到第一个unused
    pub fn find_empty_block(&self) -> Option<usize> {
        for i in 0..self.disk.fat.len() {
            if let FATItem::UnUsed = self.disk.fat[i] {
                return Some(i);
            }
        }
        None
    }

    // 分配指定数量的块，返回块号数组
    pub fn allocate_block(&mut self, cnumber: usize) -> Result<Vec<usize>, &'static str> {
        // print_info();
        // println!("Allocating {} clusters...", cnumber);

        let mut clusters: Vec<usize> = Vec::with_capacity(cnumber);
        for i in 0..cnumber {
            clusters.push(match self.find_empty_block() {
                Some(cluster) => cluster,
                _ => return Err("No enough space!"),
            });

            // print_debug();
            // println!("Allocated cluster: {}", clusters[i]);

            let cur_cluster = clusters[i];
            if i > 0 {
                self.disk.fat[clusters[i - 1]] = FATItem::Cluster(cur_cluster);
            }
            self.disk.fat[cur_cluster] = FATItem::EOF;
        }
        Ok(clusters)
    }

    // 查找某块开始的后面的块
    fn get_series(&self, start: usize) -> Result<Vec<usize>, String> {
        // print_info();
        // println!("Getting series from cluster {}...", start);

        let mut clusters: Vec<usize> = Vec::new();
        let mut cur_cluster = start;

        clusters.push(cur_cluster);
        loop {
            match self.disk.fat[cur_cluster] {
                FATItem::Cluster(cluster) => {
                    clusters.push(cluster);
                    cur_cluster = cluster;
                }
                FATItem::EOF => {
                    // print_debug();
                    // println!("Series end with {}.", cur_cluster);
                    break Ok(clusters);
                }
                FATItem::BadCluster => {
                    cur_cluster += 1;
                    continue;
                }
                _ => {
                    break Err(format!("Unexpected FATItem: {}", cur_cluster));
                }
            }
        }
    }

    // 从start删除指定块序列, 返回series
    fn delete_series(&mut self, start: usize) -> Result<Vec<usize>, String> {
        // print_info();
        // println!("Deleting series from cluster {}...", start);

        let series = self.get_series(start)?;
        let clusters = series.clone();
        for cluster in clusters {
            self.disk.fat[cluster] = FATItem::UnUsed;
        }
        Ok(series)
    }

    fn calculate_blocks_with_eof(len: usize) -> (bool, usize) {
        let mut number: f32 = len as f32 / BLOCK_SIZE as f32;
        let mut eof = false;
        if number.fract() != 0.00000 {
            number = number + 1.0;
            eof = true;
        }
        (eof, number as usize)
    }

    // 写入数据，返回数据开始块号
    pub fn write_to_disk(&mut self, data: &[u8]) -> usize {
        // print_info();
        // println!("Writing data to disk...");

        let (eof, blocks_number) = DiskOperator::calculate_blocks_with_eof(data.len());
        let clusters = self.allocate_block(blocks_number).unwrap();
        self.disk.write_in_clusters(data, clusters.as_slice(), eof);

        // print_debug();
        // println!("Data written to disk, start cluster: {}", clusters[0]);
        clusters[0]
    }

    // 在当前文件夹新建文件夹
    pub fn new_directory(&mut self, name: &str) -> Result<(), &'static str> {
        // print_info();
        // println!("Creating new directory: {}", name);
        // print_debug();
        // println!("Trying to write to disk");

        if let Some((_, _)) = self.cur_dir.get_fcb(name) {
            return Err("Directory already exists!");
        }

        // 创新新目录，添加.和..
        let mut new_dir = Directory::new(name);
        // ?
        new_dir.files.push(Fcb {
            name: String::from("."),
            file_type: FileType::Directory,
            first_cluster: self.find_empty_block().unwrap(),
            length: 0,
        });

        new_dir.files.push(Fcb {
            name: String::from(".."),
            file_type: FileType::Directory,
            first_cluster: self.cur_dir.files[0].first_cluster,
            length: 0,
        });

        // 将新目录序列化后写入磁盘
        let bin_dir = bincode::serialize(&new_dir).unwrap();

        let first_cluster = self.write_to_disk(bin_dir.as_slice());
        // print_debug();
        // println!("adding FCB to current directory...");

        self.cur_dir.files.push(Fcb {
            name: String::from(name),
            file_type: FileType::Directory,
            first_cluster,
            length: 0,
        });
        // 当前文件夹未更新数据写入磁盘，只增加了fcb，写入磁盘的操作在set_current_dir中
        // print_debug();
        // println!("Directory {} created successfully!", name);

        Ok(())
    }

    // 根据首块获得数据
    fn get_data_by_first_cluster(&self, first_cluster: usize) -> Vec<u8> {
        // print_debug();
        // println!("Getting data by clusters...");

        let clusters = self.get_series(first_cluster).unwrap();
        let data = self.disk.read_in_clusters(clusters.as_slice());

        // print_debug();
        // println!("Data read successfully!");

        data
    }

    // 通过FCB获取目录
    fn get_directory_by_fcb(&self, fcb: &Fcb) -> Directory {
        // print_debug();
        // println!("Getting directory by FCB {:?}...", fcb);

        match fcb.file_type {
            FileType::Directory => {
                let data = self.get_data_by_first_cluster(fcb.first_cluster);
                // 反序列化
                let dir = bincode::deserialize(data.as_slice()).unwrap();
                dir
            }
            _ => panic!("Not a directory!"),
        }
    }

    // 通过FCB获取文件
    fn get_file_by_fcb(&self, fcb: &Fcb) -> Vec<u8> {
        // print_info();
        // println!("Getting file by FCB {:?}...", fcb);

        match fcb.file_type {
            FileType::File => self.get_data_by_first_cluster(fcb.first_cluster),
            _ => panic!("Not a file!"),
        }
    }

    // 通过FCB删除文件,先删除占用的磁盘块，再从当前文件夹删除FCB
    fn delete_file_by_fcb(&mut self, fcb: &Fcb) -> Result<(), String> {
        // print_info();
        // println!("Deleting file by FCB {:?}...", fcb);

        if let FileType::Directory = fcb.file_type {
            let dir = self.get_directory_by_fcb(fcb);
            if dir.files.len() > 2 {
                return Err("Directory is not empty!".to_string());
            }
        }

        if let Err(err) = self.delete_series(fcb.first_cluster) {
            return Err(err);
        }

        match self.cur_dir.get_fcb(fcb.name.as_str()) {
            Some(fcb_data) => {
                let index = fcb_data.0;
                self.cur_dir.files.remove(index);
            }
            None => {
                return Err("FCB not found!!".to_string());
            }
        }

        Ok(())
    }

    // 当前文件夹创建文件
    pub fn new_file(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        // print_info();
        // println!("Creating new file: {}", name);

        if self.cur_dir.get_fcb(name).is_some() {
            return Err("File already exists!".to_string());
        }

        // 写入数据
        let first_cluster = self.write_to_disk(data);
        let new_file_fcb = Fcb {
            name: String::from(name),
            file_type: FileType::File,
            first_cluster,
            length: data.len(),
        };
        self.cur_dir.files.push(new_file_fcb);
        
        // 更新文件夹大小，将写入新数据的文件夹重新写入磁盘
        let add_length = data.len();
        self.cur_dir.files[0].length += add_length;

        Ok(())
    }

    // 以文件名读取文件
    pub fn read_file_by_name(&self, name: &str) -> Result<Vec<u8>, String> {
        match self.cur_dir.get_fcb(name) {
            Some((_, fcb)) => Ok(self.get_file_by_fcb(fcb)),
            None => Err("File not found!".to_string()),
        }
    }

    pub fn delete_file_by_name(&mut self, name: &str) -> Result<(), String> {
        // print_debug();
        // println!("Deleting file by name: {}, cur_dir: {}", name,self.cur_dir.name);
        let fcb = match self.cur_dir.get_fcb(name) {
            Some((_, fcb)) => fcb.clone(),
            None => return Err("File not found!".to_string()),
        };
        let decrease_length = fcb.clone().length;
        self.cur_dir.files[0].length -= decrease_length;
        let _ = self.delete_file_by_fcb(&fcb);

        Ok(())
    }

    // 将文件夹保存至磁盘，返回初始块号
    fn save_dir_to_disk(&mut self, dir: &Directory) -> usize {
        // print_debug();
        // println!("Saving directory to disk...");

        let data = bincode::serialize(dir).unwrap();
        let (eof, blocks_number) = DiskOperator::calculate_blocks_with_eof(data.len());
        // 重新分配块
        let _ = self.delete_series(self.cur_dir.files[0].first_cluster);

        let clusters = self.allocate_block(blocks_number).unwrap();
        self.disk
            .write_in_clusters(data.as_slice(), clusters.as_slice(), eof);

        clusters[0]
    }

    // 保存当前文件夹至磁盘,并以文件夹名称切换当前文件夹
    pub fn set_current_dir(&mut self, name: &str) {
        let dir = self.cur_dir.clone();
        self.save_dir_to_disk(&dir);

        if name == ".." {
            let size = self.cur_dir.files[0].length.clone();
            let cur_name = self.cur_dir.name.clone();
            let (_, fcb) = self.cur_dir.get_fcb(name).unwrap();
            self.cur_dir = self.get_directory_by_fcb(fcb);
            
            let (index, _) = self.cur_dir.get_fcb(&cur_name).unwrap();
            self.cur_dir.files[index].length = size;
        }
        else {
            let (_, fcb) = self.cur_dir.get_fcb(name).unwrap();
            self.cur_dir = self.get_directory_by_fcb(fcb);
        }

    }

    // 更改文件名
    pub fn rename_file(&mut self, old: &str, new: &str) {
        if let Some((index, fcb)) = self.cur_dir.get_fcb(old) {
            let new_fcb = Fcb {
                name: String::from(new),
                ..fcb.to_owned()
            };
            self.cur_dir.files[index] = new_fcb;
        } else {
            println!("FCB not found for name: {}", old);
        }
    }

    // 获取磁盘大小，已分配，未分配数量
    pub fn get_disk_info(&self) -> (usize, usize, usize) {
        let disk_size = self.disk.fat.len();
        let mut used = 0;
        let mut unused = 0;
        for item in &self.disk.fat {
            match item {
                FATItem::UnUsed => unused += 1,
                FATItem::BadCluster => continue,
                _ => used += 1,
            }
        }

        (disk_size, used, unused)
    }

    // 复制文件
    pub fn copy_file_by_name(&mut self, name: &str, path: &str) {
        let (fcb, _) = match self.cur_dir.get_fcb(name) {
            Some((index, fcb)) => (fcb.clone(), index),
            None => {
                println!("File not found!");
                return;
            }
        };

        // 通过路径找到目标文件夹
        let path: Vec<&str> = path.split('/').collect();
        let mut cur_dir = self.cur_dir.clone();
        for dir in path {
            if dir == "" {
                continue;
            }
            else {
                let (_, fcb1) = cur_dir.get_fcb(dir).unwrap();
                cur_dir = self.get_directory_by_fcb(fcb1);
            }
        }
        
        // 在目标文件夹新建文件并写入数据
        let data = self.get_file_by_fcb(&fcb);
        let file_name = fcb.name.clone();
        if cur_dir.get_fcb(name).is_some() {
            println!("File already exists!");
            return;
        }
        // 写入数据
        let first_cluster = self.write_to_disk(data.as_slice());
        let new_file_fcb = Fcb {
            name: String::from(file_name),
            file_type: FileType::File,
            first_cluster,
            length: data.len(),
        };
        cur_dir.files.push(new_file_fcb);
        
        // 将写入新数据的文件夹重新写入磁盘
        let data = bincode::serialize(&cur_dir).unwrap();
        let (eof, blocks_number) = DiskOperator::calculate_blocks_with_eof(data.len());
        self.delete_series(cur_dir.files[0].first_cluster).unwrap();
        let clusters = self.allocate_block(blocks_number).unwrap();
        self.disk.write_in_clusters(data.as_slice(), clusters.as_slice(), eof);

    }

    pub fn move_file_by_name(&mut self, name: &str, path: &str) {
        let (fcb, index) = match self.cur_dir.get_fcb(name) {
            Some((index, fcb)) => (fcb.clone(), index),
            None => {
                println!("File not found!");
                return;
            }
        };
        self.cur_dir.files.remove(index);
        self.save_dir_to_disk(&self.cur_dir.clone());

        // 通过路径找到目标文件夹
        let path: Vec<&str> = path.split('/').collect();
        let mut cur_dir = self.cur_dir.clone();
        for dir in path {
            if dir == "" {
                continue;
            }
            else {
                let (_, fcb1) = cur_dir.get_fcb(dir).unwrap();
                cur_dir = self.get_directory_by_fcb(fcb1);
            }
        }

        if cur_dir.get_fcb(name).is_some() {
            self.cur_dir.files.push(fcb.clone());
            self.save_dir_to_disk(&self.cur_dir.clone());
            println!("File already exists!");
            return;
        }

        let add_length = fcb.length;
        cur_dir.files[0].length += add_length;
        // 将文件FCB添加至目标文件夹
        cur_dir.files.push(fcb.clone());
        let data = bincode::serialize(&cur_dir).unwrap();
        let (eof, blocks_number) = DiskOperator::calculate_blocks_with_eof(data.len());
        self.delete_series(cur_dir.files[0].first_cluster).unwrap();
        let clusters = self.allocate_block(blocks_number).unwrap();
        self.disk.write_in_clusters(data.as_slice(), clusters.as_slice(), eof);

    }

    // 输出当前绝对路径
    pub fn get_abs_path(&self) -> String {
        let mut path = String::from("");
        let mut cur_dir = self.cur_dir.clone();
        while cur_dir.name != "root" {
            path = format!("/{}/{}", cur_dir.name, path);
            let (_, fcb) = cur_dir.get_fcb("..").unwrap();
            cur_dir = self.get_directory_by_fcb(fcb);
        }
        path = format!("/root{}", path);
        if path.chars().last() == Some('/') {
            path.pop();
        }
        path
    }
}
