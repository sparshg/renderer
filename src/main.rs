use pollster;
use webg::run;

fn main() {
    pollster::block_on(run());
}
