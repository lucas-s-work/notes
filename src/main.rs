use notes::view::View;

mod notes;

fn main() {
    let mut view = View::new().expect("failed to load or create notes view");
    match view.render() {
        Err(e) => println!("Encountered error: {:?}", e),
        _ => (),
    };
    view.save().expect("failed to save notes");
}
