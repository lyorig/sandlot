use sandlot::{
    Result, audio,
    init::{Audio, Context},
};

fn run() -> Result<()> {
    for i in 0..audio::num_drivers() {
        println!("{}", audio::driver(i).unwrap());
    }

    let ctx = Context::init()?;
    let _aud = Audio::leak(ctx.as_ref())?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An error occurred: {e}");
    }
}
