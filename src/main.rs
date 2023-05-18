mod recipe;

#[cfg(all(feature = "gui", feature = "term"))]
compile_error!("Can only have `gui` *or* `term` enabled at once");

#[cfg(not(any(feature = "gui", feature = "term")))]
compile_error!("Must have `gui` *or* `term` enabled");

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "term")]
mod term;

#[cfg(feature = "gui")]
#[macroquad::main("Little Chemistry")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    gui::main().await
}

#[cfg(feature = "term")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    term::main()
}
