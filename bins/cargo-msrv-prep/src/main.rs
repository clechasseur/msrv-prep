use cargo_msrv_prep::hello_world_in_bin;
use msrv_prep_lib::hello_world;

fn main() {
    println!("{}", hello_world());
    println!("{}", hello_world_in_bin());
}
