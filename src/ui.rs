use crate::disk_operator;
use disk_operator::*;
use crate::disk::BLOCK_SIZE;

use std::{fs, io::{stdin, stdout, Write}};

pub fn load_ui(filename: &str) -> DiskOperator {
    let mut buffer = String::new();
    loop {
        print_info();
        print!("Load file-system.vd? [y/n]: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let input = buffer.as_str().trim().chars().next().unwrap();

        match input {
            'Y' | 'y' => {
                println!();
                println!("Loading file-system.vd...");
                let data = fs::read(filename).unwrap();
                break bincode::deserialize(data.as_slice()).unwrap();
            },
            'N' | 'n' => {
                print_info();
                println!("Creating new file-system.vd...");
                break DiskOperator::new(None);
            },
            _ => {
                println!("Invalid input, please try again.");
                continue;
            }
        }
    }
}

const UI_INIT: &str = "\
\n==================================================\
\n           Simple File System\
\n==================================================\
\nHelp:\
\n\tcd <dirname>: Change current dir.\
\n\tmkdir <dir name>: Create a new dir.\
\n\tls : List all files and dir in current dir.\
\n\tcat <filename>: Show the file content.\
\n\trm <filename>: Delete a file on disk.\
\n\tdiskinfo : Show some info about disk.\
\n\tsave : Save this virtual disk to file 'file-sys.vd'\
\n\texit : Exit the system. 
\n\
\nTesting:\
\n\ttest create: Create a random file to test.\
\n\
\nSystem Inner Function:\
\n\tfn create_file_with_data(&mut self, name: &str, data: &[u8])\
\n\tfn rename_file(&mut self, old: &str, new: &str)\
\n\tfn delete_file_by_name(&mut self, name: &str)\
\n\tfn read_file_by_name(&self, name: &str) -> Vec<u8>\
\n"; // UI主菜单

pub const SAVE_FILE_NAME: &str = "./file-system.vd";

pub fn interact_with_user(vd: &mut DiskOperator) {
    println!("{}", UI_INIT);
    
    let mut input = String::new();
    loop {
        input.clear();
        print!("$ ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let mut args = String::from(input.trim());
        
        if args.starts_with("help") {
            println!("{}", UI_INIT);
        }
        else if args.starts_with("exit") {
            print_info();
            println!("Exiting...");
            break;
        }
        else if args.starts_with("save") {
            print_info();
            println!("Saving file-system.vd...");
            let data = bincode::serialize(&vd).unwrap();
            fs::write(SAVE_FILE_NAME, data.as_slice()).unwrap();
            print_info();
            println!("File saved.");
        }
        else if let Some(name) = args.strip_prefix("cd ") {
            print_info();
            println!("Changing dir to {}...", name);
            vd.set_current_dir(name);
        }
        else if let Some(name) = args.strip_prefix("mkdir ") {
            print_info();
            println!("Creating dir {}...", name);
            vd.new_directory(name).unwrap();
        }
        else if args.starts_with("ls") {
            println!("{}", vd.cur_dir);
        }
        else if let Some(name) = args.strip_prefix("cat ") {
            let data = vd.read_file_by_name(name.trim()).unwrap();
            println!("{}", String::from_utf8(data).unwrap());
        }
        else if let Some(name) = args.strip_prefix("rm ") {
            vd.delete_file_by_name(name.trim()).expect("Error: Delete Failed.");
            print_info();
            println!("File {} deleted.", name);
        }
        else if args.starts_with("diskinfo") {
            let (disk_size, used_size, unused_size) = vd.get_disk_info();
            println!("Disk Size: {} bytes", disk_size * BLOCK_SIZE);
            println!("Used Size: {} bytes", used_size * BLOCK_SIZE);
            println!("Unused Size: {} bytes", unused_size * BLOCK_SIZE);
        }
        else {
            println!("Invalid command, please try again.");
        }

    }
}