extern crate glutin;
extern crate glium;
extern crate test;

use glium::DisplayBuild;

#[bench]
fn initialization(bencher: &mut test::Bencher) {
    bencher.bench_n(1, |bencher| {
        bencher.iter(|| {
            glutin::WindowBuilder::new().with_visibility(false).build_glium().unwrap()
        });
    });
}
