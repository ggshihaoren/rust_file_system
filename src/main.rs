mod ui;
use ui::*;
pub mod disk;
pub mod disk_operator;
extern crate lazy_static;

fn main() {

    // unsafe {
    //     let mut vd = load_ui(VIRTUAL_DISK_NAME);
    //     interact_with_user(&mut vd);
    // }
    let mut vd = load_ui();
    interact_with_user(&mut vd);
    
}

// TODO: 1. 自定义文件系统名称(solved) 2. 文件的length设定  3. mv 的路径需要'/' (solved)
//       4. cp和mv文件的first_cluster是同一个，需要重新分配(solved)
//       5. 写上当前路径()