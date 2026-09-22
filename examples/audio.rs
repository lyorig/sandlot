use sandlot::audio;

fn main() {
    for i in 0..audio::num_drivers() {
        println!("{}", audio::driver(i).unwrap());
    }
}
