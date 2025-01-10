use boa_engine::{Context, Source};

mod wasi;
use wasi::inject_shims;

#[ic_cdk::query]
fn eval(s: String) -> String {
    let mut ctx = Context::default();

    ctx.eval(Source::from_bytes(&s))
        .unwrap()
        .to_string(&mut ctx)
        .unwrap()
        .to_std_string_escaped()
}

#[ic_cdk::init]
fn init_fn() {
    inject_shims();
}
