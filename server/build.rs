use vergen_git2::{BuildBuilder, Emitter, Git2Builder};

fn main() {
    let build = BuildBuilder::all_build().unwrap();
    let git2 = Git2Builder::all_git().unwrap();

    Emitter::default()
        .add_instructions(&build)
        .unwrap()
        .add_instructions(&git2)
        .unwrap()
        .emit()
        .unwrap();
}
