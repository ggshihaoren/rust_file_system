use crate::disk_operator;
use disk_operator::*;
use crate::disk::BLOCK_SIZE;

use std::{fs, io::{stdin, stdout, Write}};
use lazy_static::lazy_static;
use std::sync::Mutex;



// pub static mut VIRTUAL_DISK_NAME: &str = "./file-system.vd";
lazy_static! {
    pub static ref VIRTUAL_DISK_NAME: Mutex<&'static str> = Mutex::new("./file-system.vd");
}

pub fn load_ui() -> DiskOperator {
    let mut buffer = String::new();
    loop {
        print_info();
        print!("Load specified file-system.vd? [y/n]: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let input = buffer.as_str().trim().chars().next().unwrap();

        match input {
            'Y' | 'y' => {
                print_info();
                print!("Input the name of your virtual disk: ");
                stdout().flush().unwrap();
                let mut filename = String::new();
                stdin().read_line(&mut filename).unwrap();
                let mut disk_name = VIRTUAL_DISK_NAME.lock().unwrap();
                *disk_name = Box::leak(filename.trim().to_string().into_boxed_str());
                let data = fs::read(*disk_name).unwrap();
                print_info();
                println!("Loading {}...", *disk_name);
                break bincode::deserialize(data.as_slice()).unwrap();
            },
            'N' | 'n' => {
                print_info();
                print!("Input the name of new virtual disk: ");
                stdout().flush().unwrap();
                let mut filename = String::new();
                stdin().read_line(&mut filename).unwrap();
                // unsafe {
                    // VIRTUAL_DISK_NAME = Box::leak(buffer.trim().to_string().into_boxed_str());
                // }
                let mut disk_name = VIRTUAL_DISK_NAME.lock().unwrap();
                *disk_name = Box::leak(filename.trim().to_string().into_boxed_str());
                print_debug();
                println!("Creating new {}...", *disk_name);
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
\n         Simple File System in Rust\
\n==================================================\
\nCommands:\
\n\tcd <dirname>: Change current dir.\
\n\tmkdir <dirname>: Create a new dir.\
\n\tls : List all files and dir in current dir.\
\n\ttouch <filename> <data>: Create a new file.\
\n\tcat <filename>: Show the file content.\
\n\trm <filename>: Delete a file on disk.\
\n\tcp <src> <dst>: Copy a file.\
\n\tmv <src> <dst>: Move a file.\
\n\tdiskinfo : Show some info about disk.\
\n\tsave : Save this virtual disk to file.\
\n\texit : Exit the system. 
\n"; // UI主菜单

pub fn interact_with_user(vd: &mut DiskOperator) {
    println!("{}", UI_INIT);
    
    let mut input = String::new();
    loop {
        input.clear();
        print!("$ ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let args = String::from(input.trim());
        
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
            println!("Saving {}...", VIRTUAL_DISK_NAME.lock().unwrap().clone());
            let data = bincode::serialize(&vd).unwrap();
            // unsafe {
            //     fs::write(VIRTUAL_DISK_NAME, data.as_slice()).unwrap();
            // }
            fs::write(VIRTUAL_DISK_NAME.lock().unwrap().clone(), data.as_slice()).unwrap();
            print_info();
            println!("File saved.");
        }
        else if let Some(name) = args.strip_prefix("cd ") {
            // print_info();
            // println!("Changing dir to {}...", name);
            vd.set_current_dir(name);
        }
        else if let Some(name) = args.strip_prefix("mkdir ") {
            // print_info();
            // println!("Creating dir {}...", name);
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
            // print_info();
            // println!("File {} deleted.", name);
        }
        else if args.starts_with("diskinfo") {
            let (disk_size, used_size, unused_size) = vd.get_disk_info();
            println!("Disk Size: {} bytes", disk_size * BLOCK_SIZE);
            println!("Used Size: {} bytes", used_size * BLOCK_SIZE);
            println!("Unused Size: {} bytes", unused_size * BLOCK_SIZE);
        }
        else if let Some(name) = args.strip_prefix("cp ") {
            let name: Vec<&str> = name.split(" ").collect();
            if name.len() != 2 {
                println!("Invalid command, please try again.");
                continue;
            }
            vd.copy_file_by_name(name[0], name[1]);
        }
        else if let Some(name) = args.strip_prefix("mv ") {
            let name: Vec<&str> = name.split(" ").collect();
            if name.len() != 2 {
                println!("Invalid command, please try again.");
                continue;
            }
            // 移动与重命名
            if name[1].contains("/") {
                vd.move_file_by_name(name[0], name[1]);
            }
            else {
                if let FileType::Directory = vd.cur_dir.get_file_type(name[1]).unwrap() {
                    vd.move_file_by_name(name[0], name[1]);
                }
                else {
                    vd.rename_file(name[0], name[1]);
                }
            }
        }
        else if let Some(name) = args.strip_prefix("cat ") {
            let data = vd.read_file_by_name(name.trim()).unwrap();
            println!("{}", String::from_utf8(data).unwrap());
        }
        else if let Some(name) = args.strip_prefix("touch ") {
            let mut index = 0;
            for i in name.trim().chars() {
                if i == ' ' {
                    break;
                }
                index += 1;
            }
            let file_name = &name.trim()[..index];
            let mut data = String::from(&name.trim()[index+1..]);
            let time: String = format!("\nGnerated at {:?}.", chrono::Local::now());
            data.push_str(&time);
            vd.new_file(file_name, data.as_bytes()).unwrap();
        }
        else {
            println!("Invalid command, please try again.");
        }

    }
}