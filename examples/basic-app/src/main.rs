use basic_app::run;

fn main() {
    pollster::block_on(run());
}
