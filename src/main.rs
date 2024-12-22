mod ui;
use ui::*;
pub mod disk;
pub mod disk_operator;

fn main() {

    let mut vd = load_ui(SAVE_FILE_NAME);
    interact_with_user(&mut vd);
    
}
