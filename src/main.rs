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