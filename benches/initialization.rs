extern crate glutin;
extern crate glium;
extern crate test;

mod support;

#[bench]
fn initialization(bencher: &mut test::Bencher) {
    bencher.bench_n(1, |bencher| {
        bencher.iter(|| {
    		support::build_display()
        });
    });
}
