


struct FileInfo {
    pub name: String,
    created_at: DateTime<Local>,
    updated_at: DateTime<Local>,
    abs_path: String,
    size: u64
}

pub enum FileNode {
    File {
        file_info: FileInfo,
        content: String
    },
    Directory {
        file_info: FileInfo,
        files: Vec<FileNode>
    }
}

pub fn generate_dir(name: String, path: String) ->FileNode {
    let file = FileInfo {
        name: name.clone(),
        created_at: Local::now(),
        updated_at: Local::now(),
        abs_path: format!("{}/{}", path, name),
        size: 0
    };
    return FileNode::Directory { file_info: file, files: vec![] }
}

pub fn generate_file(name: String, path: String, content: String) -> FileNode {
    let file = FileInfo {
        name: name.clone(),
        created_at: Local::now(),
        updated_at: Local::now(),
        abs_path: format!("{}/{}", path, name),
        size: 0
    };
    return FileNode::File { file_info: file, content: content }
}

impl FileNode {
    pub fn add_file(&mut self, file: FileNode) -> Option<&mut FileNode> {
        match self {
            FileNode::File {..} => None,
            FileNode::Directory { files, ..} => {
                files.push(file);
                return Some(files.last_mut().unwrap());
            }
        }
    }

    pub fn ls(&self, prefix: String) {
        if prefix == "" {    // 根目录
            println!("path                type                size                todo");
            println!("----------------------------------------------------------------");
        }
        match self {
            FileNode::File { file_info, .. } => {
                let full_path = format!("{}{}", prefix, file_info.name);
                println!("{:<20}file", full_path);
            },
            FileNode::Directory {file_info, files } => {
                let full_path = format!("{}{}", prefix.clone(), file_info.name);
                println!("{:<20}directory", full_path);
                for file in files {
                    if prefix == "" {    // 根目录
                        file.ls(String::from("/root/"));
                    } else {
                        file.ls(format!("{}{}/", prefix, file_info.name));
                    }
                }
            }
        }
    }
}