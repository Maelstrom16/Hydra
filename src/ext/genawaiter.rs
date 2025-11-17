#[macro_export]
macro_rules! gen_all {
    ($co:expr, $closure:expr) => {{
        let mut shelf_step = genawaiter::stack::Shelf::new();
        let mut co_step = unsafe { genawaiter::stack::Gen::new(&mut shelf_step, $closure) };

        let result = {
            loop {
                match co_step.resume() {
                    genawaiter::GeneratorState::Yielded(_) => $co.yield_(()).await, // Propagate yield
                    genawaiter::GeneratorState::Complete(value) => {
                        drop(co_step);
                        break value;
                    }
                }
            }
        };

        result
    }};
}
